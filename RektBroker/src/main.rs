// This document contain the main task of the broker. The task datagram_handler
// must never be blocked by any method ! The whole project use tokio and work
// asynchronously to allow a maximum bandwidth computing. The goal of the broker is
// to handle 100Gb/s.
// For each feature the memory management and the cpu usage should be in the middle of the reflexion.
//
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 21/10/2023\

#![allow(unused)] // remove this line in production
#![allow(unused_doc_comments)]

#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate rekt_lib;

use std::io::Bytes;
use std::net::{IpAddr, SocketAddr};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crossbeam_queue::ArrayQueue;
use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use lazy_static::lazy_static;
use local_ip_address::local_ip;
use parking_lot::{Condvar, Mutex};
use quinn::{Connecting, Connection, ConnectionError, Endpoint, ServerConfig};
use quinn::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use quinn::rustls::pki_types::pem::PemObject;
use rcgen::CertifiedKey;
use rustls::{Certificate, PrivateKey};
use serde::Serialize;
use tokio::net::UdpSocket;
use tokio::task::JoinHandle;
use tokio::{join, task, try_join};

use crate::clients::client::{Client, ConnectionId, Packet};
use crate::config::Config;
use crate::errors::Error;
use crate::errors::Error::InitializationError;
use crate::prelude::{ClientMap, Result};
use crate::streams::streams::RBiStream;

mod clients;
mod config;
mod errors;
mod job_system;
mod prelude;
mod streams;

lazy_static! {
    // Global config and general purpose vars
    static ref SERVER_IS_RUNNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    static ref CONFIG: Config = Config::new(); // Unique reference to the config object

    // Client vars
    static ref CLIENT_MAP: ClientMap = Arc::new(DashMap::default()); // store each client connection <ConnectionId, Client>

    // Job system vars
    static ref PACKET_BUFFER: Arc<ArrayQueue<Packet>> = Arc::new(ArrayQueue::new(CONFIG.packet_buffer_size.into()));
    static ref WORKER_CONDVAR: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(false), Condvar::new()));

/*
    // List of client's :
    static ref CLIENTS_SENDERS_REF: ClientsHashMap<ClientSender> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, Sender> -> the sender is used to sent command through the mpsc channels
    static ref CLIENTS_STRUCTS_REF: ClientsHashMap<Arc<Mutex<Client>>> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, Struct> -> used only to keep struct alive
    static ref CLIENTS_ADDRESSES_REF: ClientsHashMap<SocketAddr> = Arc::new(RwLock::new(HashMap::default())); // <Client ID, address> -> Used to send data

    // List of time reference for ping requests
    static ref PINGS_REF: PingsHashMap = Arc::new(Mutex::new(HashMap::default())); // <Ping ID, reference time in ms>

    // List of topic subscribers
    static ref TOPICS_SUBSCRIBERS_REF: TopicsHashMap<HashSet<ClientId>> = Arc::new(RwLock::new(HashMap::default())); // <Topic ID, [Clients ID]>

    // List of Objects (group of topics)
    static ref OBJECTS_TOPICS_REF: ObjectHashMap<HashSet<TopicId>> = Arc::new(RwLock::new(HashMap::default())); // <ObjectId, [TopicId]>
    static ref OBJECT_SUBSCRIBERS_REF: ObjectHashMap<HashSet<ClientId>>  = Arc::new(RwLock::new(HashMap::default())); // <ObjectId, [ClientId]>*/
}

#[tokio::main]
async fn main() {
    // Set the rust log environment variable and then init the rust logger
    std::env::set_var("RUST_LOG", &CONFIG.debug_level);
    pretty_env_logger::init();

    info!("Static variables and configuration initialized ...");
    info!("Log level set to {} ...", &CONFIG.debug_level);
    info!("Check config.toml file to change the config.");

    // ----------------------------------------------------
    // Starting the server
    // ----------------------------------------------------
    info!("Starting the server :");

    let endpoint_handle = tokio::spawn(async {
        open_endpoint().await;
    });
    let job_system_handle = tokio::spawn(async {
        job_system::init_job_system().await;
    });

    let handles_results = try_join!(endpoint_handle, job_system_handle);

    match handles_results {
        Ok(_) => {}
        Err(err) => {
            error!("{}", err);
            error!(">>> Server stopping!");
            return;
        }
    }
}

fn init_quic_connection() -> Result<(ServerConfig)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let key = PrivateKeyDer::from_pem_reader(&mut cert.key_pair.serialize_pem().as_bytes())?;
    let server_config =
        ServerConfig::with_single_cert(vec![cert.cert.der().clone()], key)?;

    Ok((server_config))
}

async fn open_endpoint() -> Result<()> {
    let mut quic_config = match init_quic_connection() {
        Ok(quic_config) => {
            info!("- QUIC connection setup !");
            quic_config
        }
        Err(err) => {
            return Err(InitializationError(format!(
                "Quic initialization failed : {}",
                err.to_string()
            )));
        }
    };

    // Fetch the local ip address and config port to start the server
    let addr: SocketAddr = match local_ip() {
        Ok(ip) => {
            info!("Server starting on {}:{} ...", ip, &CONFIG.port);
            format!("127.0.0.1:{}", &CONFIG.port).parse()?
        }
        Err(err) => {
            // can't start the server if the local ip can't be reach
            return Err(InitializationError(format!(
                "Failed to get local IP:\n{}",
                err
            )));
        }
    };

    // Bind this endpoint to a UDP socket on the given server address.
    let endpoint = Endpoint::server(quic_config, addr)?;

    // Start iterating over incoming connections.
    while let Some(incoming) = endpoint.accept().await {
        let connecting = match incoming.accept() {
            Ok(conn) => {conn}
            Err(e) => {
                error!("Connection failed : {}", e);
                continue;
            }
        };
        let connection_process = handle_connection(connecting);
        tokio::spawn(async move {
            connection_process.await;
        });
    }

    Ok(())
}

async fn handle_datagram(packet: Packet) {
    // TODO : Handle packet according the source and the datagram

    // 1 - fetch a ref ot the client :
    let client = match CLIENT_MAP.get_mut(&packet.source) {
        None => {
            return;
        }
        Some(entry) => entry,
    };

    // 2 - build the datagram struct + respond to it

    client
        .unreliable_stream
        .stream
        .send_datagram(bytes::Bytes::from("Un Packet unreliable"));
}

async fn handle_connection(pending_connection: Connecting) -> Result<()> {
    // wait for connection handshake
    let mut connection = match pending_connection.await {
        Ok(conn) => conn,
        Err(error) => {
            error!("New connection attempted but failed. Error : {}", error);
            return Err(error.into());
        }
    };

    // Open the bidirectional stream to this client
    let connection_id = ConnectionId::from_connection(&connection);
    info!(
        "New connection received from {}\nOpening a bi-stream to this connection",
        connection_id
    );
    let (mut sender, mut receiver) = connection.open_bi().await?;

    // Store the client to the static hashmap.
    let client = Client::new(connection_id, connection, RBiStream { sender, receiver });
    CLIENT_MAP.entry(connection_id).insert(client);

    Ok(())
}
