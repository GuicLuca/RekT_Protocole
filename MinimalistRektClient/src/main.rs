#![allow(unused)]

use log::{error, info, log, trace, warn};
use quinn::crypto::rustls::QuicClientConfig;
use quinn::rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified};
use quinn::rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use quinn::rustls::{DigitallySignedStruct, SignatureScheme};
use quinn::{ClientConfig, Connection, Endpoint, SendStream};
use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use lazy_static::lazy_static;
use rekt_lib::datagrams::connect_requests::{DtgConnect, DtgConnectAck, DtgConnectNack};
use rekt_lib::datagrams::data_request::DtgData;
use rekt_lib::datagrams::heartbeat_requests::{DtgHeartbeat, DtgHeartbeatRequest};
use rekt_lib::datagrams::latency_requests::{DtgPing, DtgPong};
use rekt_lib::datagrams::miscellaneous_requests::{DtgServerStatus, DtgServerStatusACK};
use rekt_lib::datagrams::object_requests::{DtgObjectRequest, DtgObjectRequestACK, DtgObjectRequestNACK};
use rekt_lib::datagrams::shutdown_request::DtgShutdown;
use rekt_lib::datagrams::topic_request::{DtgTopicRequest, DtgTopicRequestAck, DtgTopicRequestNack};
use rekt_lib::enums::datagram_type::{display_datagram_type, DatagramType};
use rekt_lib::enums::datagram_type::DatagramType::*;
use rekt_lib::libs::types::ClientId;
use rekt_lib::libs::utils::get_u16_at_pos;
use rekt_lib::rekt_common_ffi::CDtgObjectRequestACK;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

#[macro_use]
extern crate pretty_env_logger;

static PAYLOAD_SIZE: usize = 1024;

pub struct ClientData {
    connection_id: ClientId,
    sender: Option<Arc<RwLock<SendStream>>>,
    receiver: Option<quinn::RecvStream>,
    heartbeat_period: Option<Duration>,
}

lazy_static! {
    static ref CLIENT_DATA: Arc<RwLock<ClientData>> = Arc::new(RwLock::new(ClientData {
        connection_id: 0,
        sender: None,
        receiver: None,
        heartbeat_period: None,
    }));
    static ref CLIENT_IS_RUNNING: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    std::env::set_var("RUST_LOG", "info");
    pretty_env_logger::init();

    info!("Starting client: ");

    let mut crypto = quinn::rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let mut client_config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));

    info!("Client configuration successfully set.");

    match client(client_config).await {
        Ok(_) => {
            info!("Client connexion successfully closed.");
        }
        Err(err) => {
            error!("Client has crash due to the following error : {}", err);
            return Err(err);
        }
    };

    Ok(())
}

async fn client(config: ClientConfig) -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:6666";
    // Bind this endpoint to a UDP socket on the given client address.
    let mut endpoint = Endpoint::client(SocketAddr::from_str(addr)?)?;
    endpoint.set_default_client_config(config);

    // Connect to the server passing in the server name which is supposed to be in the server certificate.
    let connection = endpoint
        .connect(SocketAddr::from_str("127.0.0.1:3838")?, "localhost")?
        .await?;

    info!("Connected to the server: {}.", addr);

    // Start transferring, receiving data, see data transfer page.
    // TODO : Implementing stream management on server + test message transfer
    // TODO : Implementing disconnection
    // TODO : Implementing heartbeat + ping
    // TODO : Stress test 1
    tokio::spawn(async move {
        match open_bidirectional_stream(&connection).await {
            Ok(_) => {
                info!("Bidirectional stream successfully closed.");
            }
            Err(err) => {
                error!("Bidirectional stream has crash due to the following error : {:?}", err);
            }
        }
    });
    
    
    
    let mut arc_client_sender = None;
    let mut arc_client_heartbeat = None;
    
    info!("Starting the heartbeat task...");
    while CLIENT_IS_RUNNING.load(std::sync::atomic::Ordering::Acquire) {
        // Once the sender is set, send heartbeat until the client close
        {
            let client_read = &CLIENT_DATA.read().await;
            arc_client_sender = client_read.sender.clone();
            arc_client_heartbeat = client_read.heartbeat_period;
        }
        
        if let Some(sender) = arc_client_sender.clone() {
            let dtg = DtgHeartbeat::new();
            let mut sender_lock = sender.write().await;
            sender_lock.write_all(&dtg.as_bytes()).await?;
            drop(sender_lock);
        }
        
        let sleep_duration = {
            if let Some(heartbeat_period) = arc_client_heartbeat.clone() {
                heartbeat_period
            } else {
                Duration::from_secs(1) // Default value
            }
        };
        
        tokio::time::sleep(sleep_duration).await;
    }
    

    Ok(())
}

async fn open_bidirectional_stream(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let (mut send, mut recv) = connection.open_bi().await?;
    
    // Store the send stream in the global variable
    {
        let mut client_write = CLIENT_DATA.write().await;
        client_write.sender = Some(Arc::new(RwLock::new(send)));
    }

    // Spawn a new task to handle the incoming messages
    tokio::spawn(async move {
        handle_bistream_incoming_msg(&mut recv).await;
    });
    
    // send a DtgConnect
    let dtg = DtgConnect::new();
    {
        if let Some(sender) = &CLIENT_DATA.read().await.sender {
            let mut sender = sender.write().await;
            sender.write_all(&dtg.as_bytes()).await?;
            
        }
    }

    tokio::time::sleep(Duration::from_secs_f32(200.0)).await;

    Ok(())
}

async fn handle_bistream_incoming_msg(recv: &mut quinn::RecvStream) -> Result<(), Box<dyn Error>> {
    let mut client_buf: Vec<u8> = Vec::new();

    'handling: loop {
        // Stop the task if the client is closing
        if !CLIENT_IS_RUNNING.load(std::sync::atomic::Ordering::Acquire) {
            break 'handling;
        }
        
        let mut network_buf: [u8; 1524] = [0; 1524]; // 1500 bytes + 24 bytes for the QUIC header
        
        {
            match recv.read(&mut network_buf).await? {
                Some(received) => {
                    // Handle the received bytes
                    trace!("Received {} bytes from server", received);
                    client_buf.extend_from_slice(&network_buf[..received]);
                }
                None => {
                    // The stream has been closed
                    info!("Server has closed the bidirectional stream",);
                    return Ok(());
                }
            }
        } // End of the lock on the receiver stream

        let dtg_type = DatagramType::from(client_buf[0]);

        // Check if the datagram type is valid :
        if dtg_type == Unknown {
            error!(
                    "Server sent an unknown datagram type. Got \"{}\"",
                    client_buf[0]
                );
            return Err(format!("Invalid datagram type received. Got \"{}\".", client_buf[0]).into());
        }

        // Drain the whole buffer :
        let bytes_to_drain: usize = {
            if dtg_type.is_sized_datagram() {
                // the "?" will never throw an error here
                dtg_type.get_default_byte_size()
                    + get_u16_at_pos(&client_buf, 1).or_else(|_| Ok::<u16, Box<dyn Error>>(0)).unwrap() as usize
            } else {
                dtg_type.get_default_byte_size()
            }
        };

        // Ensure that the buffer is big enough to drain the bytes
        if client_buf.len() < bytes_to_drain {
            // Not enough bytes received yet, continue the loop to collect more bytes
            continue 'handling;
        }

        // drain "bytes_to_drain" bytes from the client buffer
        let datagram_bytes: Vec<u8> = client_buf.drain(..bytes_to_drain).collect();

        // Display received message
        match dtg_type {
            Connect => {
                let dtg = DtgConnect::try_from(datagram_bytes.as_slice())?;
                info!("Received a connection request from the server: {:?}", dtg);
            }
            ConnectAck => {
                let dtg = DtgConnectAck::try_from(datagram_bytes.as_slice())?;
                info!("Received a connection ack from the server: {:?}", dtg);
                
                // Store the connection id in the global variable
                {
                    let mut client_write = CLIENT_DATA.write().await;
                    client_write.connection_id = dtg.peer_id;
                    client_write.heartbeat_period = Some(Duration::from_millis(dtg.heartbeat_period as u64));
                }
                info!("Local client updated with connection id: {} and heartbeat period: {} ms.", dtg.peer_id, dtg.heartbeat_period);
            }
            ConnectNack => {
                let dtg = DtgConnectNack::try_from(datagram_bytes.as_slice())?;
                info!("Received a connection nack from the server: {:?}", dtg);
            }
            Heartbeat => {
                let dtg = DtgHeartbeat::try_from(datagram_bytes.as_slice())?;
                info!("Received a heartbeat from the server: {:?}", dtg);
            }
            HeartbeatRequest => {
                let dtg = DtgHeartbeatRequest::try_from(datagram_bytes.as_slice())?;
                info!("Received a heartbeat request from the server: {:?}", dtg);
            }
            Ping => {
                let dtg = DtgPing::try_from(datagram_bytes.as_slice())?;
                info!("Received a ping from the server: {:?}", dtg);
            }
            Pong => {
                let dtg = DtgPong::try_from(datagram_bytes.as_slice())?;
                info!("Received a pong from the server: {:?}", dtg);
            }
            Shutdown => {
                let dtg = DtgShutdown::try_from(datagram_bytes.as_slice())?;
                info!("Received a shutdown request from the server: {:?}", dtg);
            }
            ServerStatus => {
                let dtg = DtgServerStatus::try_from(datagram_bytes.as_slice())?;
                info!("Received a server status from the server: {:?}", dtg);
            }
            ServerStatusAck => {
                let dtg = DtgServerStatusACK::try_from(datagram_bytes.as_slice())?;
                info!("Received a server status ack from the server: {:?}", dtg);
            }
            TopicRequest => {
                let dtg = DtgTopicRequest::try_from(datagram_bytes.as_slice())?;
                info!("Received a topic request from the server: {:?}", dtg);
            }
            TopicRequestAck => {
                let dtg = DtgTopicRequestAck::try_from(datagram_bytes.as_slice())?;
                info!("Received a topic request ack from the server: {:?}", dtg);
            }
            TopicRequestNack => {
                let dtg = DtgTopicRequestNack::try_from(datagram_bytes.as_slice())?;
                info!("Received a topic request nack from the server: {:?}", dtg);
            }
            ObjectRequest => {
                let dtg = DtgObjectRequest::try_from(datagram_bytes.as_slice())?;
                info!("Received an object request from the server: {:?}", dtg);
            }
            ObjectRequestAck => {
                let dtg = DtgObjectRequestACK::try_from(datagram_bytes.as_slice())?;
                info!("Received an object request ack from the server: {:?}", dtg);
            }
            ObjectRequestNack => {
                let dtg = DtgObjectRequestNACK::try_from(datagram_bytes.as_slice())?;
                info!("Received an object request nack from the server: {:?}", dtg);
            }
            Data => {
                let dtg = DtgData::try_from(datagram_bytes.as_slice())?;
                info!("Received data from the server: {:?}", dtg);
            }
            _ => {
                info!("Received an unknown (or invalid) datagram type from the server: {}", display_datagram_type(dtg_type));
            }
        }
    }
    
    Ok(())
}




// UNUSED FUNCTIONS

async fn receive_bidirectional_stream(connection: &Connection) -> Result<(), Box<dyn Error>> {
    info!("Waiting for message from the server...");
    let (mut send, mut recv) = connection.accept_bi().await?;

    while let Ok(msg) = String::from_utf8(recv.read_to_end(PAYLOAD_SIZE).await?) {
        // Because it is a bidirectional stream, we can both send and receive.
        info!("Message received: {}", msg);

        send.write_all(b"response").await?;
        info!("Sent message \"response\" to the server.");
        send.finish()?;
        info!("Bidirectional stream successfully closed.");
    }

    Ok(())
}

async fn open_unidirectional_stream(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let mut send = connection.open_uni().await?;

    send.write_all(b"test").await?;
    send.finish()?;

    Ok(())
}

async fn receive_unidirectional_stream(connection: &Connection) -> Result<(), Box<dyn Error>> {
    while let Ok(mut recv) = connection.accept_uni().await {
        // Because it is a unidirectional stream, we can only receive not send back.
        println!("{:?}", recv.read_to_end(50).await?);
    }

    Ok(())
}

// Implementation of `ServerCertVerifier` that verifies everything as trustworthy.
#[derive(Debug)]
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl quinn::rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, quinn::rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, quinn::rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, quinn::rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ECDSA_NISTP521_SHA512,
        ]
    }
}
