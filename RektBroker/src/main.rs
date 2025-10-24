// This document contain the main task of the broker. The task datagram_handler
// must never be blocked by any method ! The whole project use tokio and work
// asynchronously to allow a maximum bandwidth computing.
// ~~The goal of the broker is to handle 100Gb/s. (maybe not)~~
// For each feature the memory management and the cpu usage should be in the middle of the reflexion.
//
// @author : GuicLuca (lucasguichard127@gmail.com)
// date : 21/10/2023\

#![allow(unused)] // remove this line in production
#![deny(clippy::all)] // Treat clippy warnings as errors to avoid bad practices

#[macro_use]
extern crate log;
extern crate pretty_env_logger;
extern crate rekt_lib;

use crate::clients::client::{Client, ConnectionId, Packet};
use crate::config::Config;
use crate::errors::Error;
use crate::errors::Error::Initialization;
use crate::object::Object;
use crate::prelude::{ClientMap, Result};
use crate::streams::streams::RBiStream;
use crate::topics::Topic;
use crossbeam_queue::ArrayQueue;
use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use lazy_static::lazy_static;
use local_ip_address::local_ip;
use parking_lot::{Condvar, Mutex};
use quinn::rustls::pki_types::pem::PemObject;
use quinn::rustls::pki_types::{CertificateDer, PrivateKeyDer};
use quinn::{Accept, Connecting, Connection, ConnectionError, Endpoint, ServerConfig};
use rcgen::CertifiedKey;
use rekt_lib::datagrams::data_request::DtgData;
use rekt_lib::datagrams::miscellaneous_requests::DtgServerStatusACK;
use rekt_lib::enums::datagram_type::DatagramType;
use rekt_lib::enums::end_connection_reason::EndConnexionReason;
use rekt_lib::libs::types::{ClientId, ObjectId, TopicId};
use rustls::{Certificate, PrivateKey};
use serde::Serialize;
use std::io::Bytes;
use std::net::{IpAddr, SocketAddr};
use std::ops::Deref;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use tokio::io::AsyncWriteExt;
use tokio::net::UdpSocket;
use tokio::sync::{oneshot, RwLock};
use tokio::task::JoinHandle;
use tokio::time::Sleep;
use tokio::{join, task, try_join};
use tracing::Instrument;

mod clients;
mod config;
mod errors;
mod job_system;
mod object;
mod prelude;
mod streams;
mod topics;

lazy_static! {
    // Global config and general purpose vars
    static ref SERVER_IS_RUNNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
    static ref CONFIG: Config = Config::new(); // Unique reference to the config object

    // Client vars
    static ref CLIENT_MAP: ClientMap = Arc::new(DashMap::default()); // store each client connection <ConnectionId, Client>

    // Job system vars
    static ref PACKET_BUFFER: Arc<ArrayQueue<Packet>> = Arc::new(ArrayQueue::new(CONFIG.packet_buffer_size.into()));
    static ref WORKER_CONDVAR: Arc<(Mutex<bool>, Condvar)> = Arc::new((Mutex::new(false), Condvar::new()));

    // Data related server
    static ref TOPICS: Arc<DashMap<TopicId, Topic>> = Arc::new(DashMap::new());
    static ref OBJECTS: Arc<DashMap<ObjectId, Object>> = Arc::new(DashMap::new());

    // Profiling vars
    static ref PROFILING_DATA: Arc<DashMap<String, u32>> = Arc::new(DashMap::new());
    static ref EXP_CODE: String = {
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 2 { // 1 = program name, 2 = exp code
            panic!("Usage: {} <exp_code>", args[0]);
        }

        args[1].clone()
    };
}


// #[cfg(feature = "profiling")]
fn increase_profiling_data(key: &str) {
    if PROFILING_DATA.contains_key(key) {
        let mut value = PROFILING_DATA.get_mut(key).unwrap();
        *value += 1;
    } else {
        PROFILING_DATA.insert(key.to_string(), 1);
    }
}

#[tokio::main]
async fn main() {
    // Set the rust log environment variable and then init the rust logger
    std::env::set_var("RUST_LOG", &CONFIG.debug_level);
    pretty_env_logger::env_logger::builder()
        .format_target(false)
        .init();

    info!("Static variables and configuration initialized ...");
    info!("Log level set to {} ...", &CONFIG.debug_level);
    info!("Check config.toml file to change the config.");

    // ----------------------------------------------------
    // Starting the server
    // ----------------------------------------------------
    info!("Starting the server :");

    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let endpoint_handle = tokio::spawn(async {
        open_endpoint(shutdown_rx).await;
    });
    let job_system_handle = tokio::spawn(async {
        job_system::init_job_system(shutdown_tx).await;
    });

    let handles_results = try_join!(endpoint_handle, job_system_handle);

    match handles_results {
        Ok(_) => {
            info!(">>> Server stopped successfully.");
            info!(">>> All tasks ended successfully.");
        }
        Err(err) => {
            error!("{}", err);
            error!(">>> Server stopping!");
            return;
        }
    }
}

/**
 * This method initialize the QUIC connection with the server certificate and the private key.
 * @return Result<ServerConfig> : the server configuration with the certificate and the private key.
 */
fn init_quic_connection() -> Result<(ServerConfig)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let key = PrivateKeyDer::from_pem_reader(&mut cert.key_pair.serialize_pem().as_bytes())?;
    let server_config = ServerConfig::with_single_cert(vec![cert.cert.der().clone()], key)?;

    Ok(server_config)
}

/**
 * This method open the QUIC endpoint and start listening for incoming connections.
 * This function could be considered as the main loop of the server. While the server is running, it
 * will accept incoming connections and handle them. This function is asynchronous and MUST run along the job system.
 *
 * @return Result<()> : Raise the error if the server fail a task.
 */
async fn open_endpoint(mut shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
    let mut quic_config = match init_quic_connection() {
        Ok(quic_config) => {
            info!("- QUIC connection setup successfully.");
            quic_config
        }
        Err(err) => {
            return Err(Initialization(format!(
                "Quic initialization failed : {}",
                err
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
            return Err(Initialization(format!("Failed to get local IP:\n{}", err)));
        }
    };

    // Bind this endpoint to a UDP socket on the given server address.
    let endpoint = Endpoint::server(quic_config, addr)?;

    // Start iterating over incoming connections.
    'ServerHandler: while SERVER_IS_RUNNING.load(Ordering::Acquire) {
        
        tokio::select! {
            // Accept incoming connections
            accepted = endpoint.accept() => {
                if let Some(incoming) = accepted {
                    let connecting = match incoming.accept() {
                        Ok(conn) => conn,
                        Err(e) => {
                            error!("Connection failed : {}", e);
                            continue;
                        }
                    };
                    let connection_process = handle_connection(connecting);
                    tokio::spawn(async move {
                        match connection_process.await{
                            Ok(id) => {
                                info!("Connection handler for {id} has ended successfully.");
                            }
                            Err(e) => {
                                error!("A connection handler has ended with error : {}", e);
                            }
                        }
                    });
                }else {
                    break 'ServerHandler;
                }
            }
            _ = &mut shutdown_rx => {
                info!("Stopping the endpoint ...");
                endpoint.close(0u32.into(), b"Server shutting down");
                break 'ServerHandler;
            }
        }
    }

    Ok(())
}

async fn handle_connection(pending_connection: Connecting) -> Result<ConnectionId> {
    // wait for connection handshake
    let mut connection = match pending_connection.await {
        Ok(conn) => conn,
        Err(error) => {
            error!("New connection attempted but failed. Error : {}", error);
            return Err(error.into());
        }
    };

    let span = tracing::info_span!(
        "connection",
        remote = %connection.remote_address(),
        protocol = %connection
            .handshake_data().unwrap()
            .downcast::<quinn::crypto::rustls::HandshakeData>().unwrap()
            .protocol
            .map_or_else(|| "<none>".into(), |x| String::from_utf8_lossy(&x).into_owned())
    );

    let out_id = async {
        // Open the bidirectional stream to this client
        let connection_id = ConnectionId::from_connection(&connection);
        info!("New connection established with {}", connection_id);

        // Store the client to the static hashmap.
        let client = Arc::from(RwLock::from(Client::new(connection_id, connection)));
        CLIENT_MAP.entry(connection_id).insert(client);
        info!("-> Client {} added to the client map.", connection_id);

        let client = match CLIENT_MAP.get(&connection_id) {
            None => {
                // The client has been removed from the hashmap.
                return Err(Error::MissingClient(connection_id));
            }
            Some(entry) => entry.clone(),
        };

        let client_read = client.read().await;
        let bi_stream = client_read.connection.accept_bi().await;

        match bi_stream {
            Ok((send, recv)) => {
                info!("New bidirectional stream received from {}", connection_id);
                // /!\ IMPORTANT : The lock level is converted here to prevent deadlocks by waiting on accept_bi() with a write lock
                // register the lock in the queue
                let client_mut_fut = client.write();
                // drop the read lock
                drop(client_read);
                // wait for the write lock
                let mut client_mut = client_mut_fut.await;
                client_mut.sender = Some(Arc::from(RwLock::from(send)));
                client_mut.receiver = Some(Arc::from(RwLock::from(recv)));
                drop(client_mut);
            }
            Err(e) => {
                info!("Connection {} closed for reason: {}", connection_id, e);
                // free the read lock
                drop(client_read);

                // Remove the client from the hashmap
                CLIENT_MAP.remove(&connection_id);

                info!("<- Client {} removed from the client map.", connection_id);

                return Ok(connection_id);
            }
        };

        {
            // TODO : this read lock may be as long as the client is connected so it may be a problem if need to be modified
            // TODO : The current solution is to consider client as constant and encapsulates inner fields in RwLock
            match client.clone().read().await.handle_bi_stream().await {
                Err(Error::QuinnRead { .. }) | Err(Error::QuinnReadExact { .. }) => {
                    info!("Bidirectional stream closed with client {}", connection_id);
                }
                Err(e) => {
                    error!("Error while handling bidirectional stream: {:?}", e);
                }
                _ => {}
            }
        }

        {
            // Once the stream is closed, disconnect the client
            if let Err(e) = Client::disconnect(&connection_id, EndConnexionReason::Shutdown).await {
                error!("Error while disconnecting client {}: {}", connection_id, e);
                return Err(e);
            }

            Ok(connection_id)
        }
    }
    .instrument(span)
    .await?;
    Ok(out_id)
}

/**
 * This method handle a datagram received from a client.
 * The method will fetch the client from the client map and respond to the datagram.
 *
 * @param packet : Packet : the packet received from the client.
 */
async fn handle_datagram(packet: Packet) {
    // 1 - fetch a ref ot the client :
    let client = match CLIENT_MAP.get(&packet.source) {
        None => {
            error!(
                "Can't handle datagram because client {} is not in the client map.",
                packet.source
            );
            return;
        }
        Some(entry) => entry.value().clone(),
    };

    // Get the client corresponding sender BUT do not lock it yet
    let sender = match &client.read().await.sender {
        Some(sender) => sender.clone(),
        None => {
            error!(
                "Client {} has no sender stream, so handle_datagram can't respond to the client.",
                packet.source
            );
            return;
        }
    };

    // 2 - Handle the datagram according to its type
    match DatagramType::from(packet.datagram[0]) {
        DatagramType::ServerStatus => {
            // ClientID::MAX is the maximum amount of client connected with valid ID.
            let dtg = DtgServerStatusACK::new(CLIENT_MAP.len() as ClientId);

            // send the datagram to the client
            {
                let mut sender = sender.write().await;
                let send_result = sender.write_all(&dtg.as_bytes()).await;
            }

            increase_profiling_data("ServerStatus");
        }
        DatagramType::TopicRequest => {
            Topic::handle_topic_request(packet).await;
        }
        DatagramType::ObjectRequest => {
            Object::handle_object_request(packet).await;
        }
        DatagramType::Data => {
            let dtg = match DtgData::try_from(packet.datagram.as_slice()) {
                Ok(dtg) => dtg,
                Err(e) => {
                    error!(
                        "Error while converting datagram from {} to DtgData: {}",
                        packet.source, e
                    );
                    return;
                }
            };

            // 1- find the topic
            let mut topic = match TOPICS.get_mut(&dtg.topic_id) {
                Some(topic) => topic,
                None => {
                    error!(
                        "Client {} tried to send data on an unknown topic {}.",
                        packet.source, dtg.topic_id
                    );

                    // Here some feedbacks would be nice for the client in a real implementation
                    return;
                }
            };

            // 2- publish the data on it avoiding the source of the update
            topic
                .value_mut()
                .publish(&dtg.payload, Some(packet.source))
                .await;
        }

        // Following cases are authorized to be sent in the job system but not implemented
        DatagramType::OpenStream => {
            // There is no need to implement stream for the prototype we want to test
            unimplemented!("OpenStream datagram type is not implemented.");
        }

        // Following cases MUST never be reached (handled before or not authorized)
        DatagramType::Unknown => {
            error!(
                "Handle_datagram received an unknown datagram type : {}",
                packet.datagram[0]
            );
            // return;
        }
        DatagramType::Heartbeat
        | DatagramType::HeartbeatRequest
        | DatagramType::Ping
        | DatagramType::Pong => {
            // those datagram types are not authorized to be sent in the job system and MUST be handled before in the client router (see client.rs::handle_bi_stream::direct_respond)
            trace!(
                "Datagram type {} should not be sent in the job system but only being responded directly upon reception.",
                packet.datagram[0]
            );

            tokio::spawn(async move {
                Client::direct_respond(packet.source, packet.datagram).await;
            });
            // return;
        }
        DatagramType::Connect
        | DatagramType::ConnectAck
        | DatagramType::ConnectNack
        | DatagramType::Shutdown => {
            // those datagram types are not handled in the job system, they are handled in the connection process
            trace!(
                "Datagram type {} should not be sent in the job system (Handled in the connection process).",
                packet.datagram[0]
            );
            // nothing to do here
            // return;
        }
        DatagramType::ServerStatusAck
        | DatagramType::TopicRequestAck
        | DatagramType::TopicRequestNack
        | DatagramType::ObjectRequestAck
        | DatagramType::ObjectRequestNack => {
            // those datagram types are not allowed to be sent by the client
            error!(
                "Datagram type {} is not authorized to be sent by the client.",
                packet.datagram[0]
            );
            // return;
        }
    }
}
