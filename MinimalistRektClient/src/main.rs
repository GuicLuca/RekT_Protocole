#![allow(unused)]

use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use log::{error, info};
use pretty_logger::{Destination, Theme};
use quinn::{ClientConfig, Connection, Endpoint};

static PAYLOAD_SIZE: usize = 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_logger::init(
        Destination::Stdout,
        "info".parse().unwrap(),
        Theme::default(),
    )?;

    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let client_config = ClientConfig::new(Arc::new(crypto));

    let mut iter: u64 = 0;

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
    // Bind this endpoint to a UDP socket on the given client address.
    let mut endpoint = Endpoint::client(SocketAddr::from_str("127.0.0.1:6666")?)?;
    endpoint.set_default_client_config(config);

    // Connect to the server passing in the server name which is supposed to be in the server certificate.
    let connection = endpoint
        .connect(SocketAddr::from_str("127.0.0.1:3838")?, "localhost")?
        .await?;

    // Start transferring, receiving data, see data transfer page.
    // TODO : Implementing connection process on server
    // TODO : Implementing stream management on server + test message transfer
    // TODO : Implementing disconnection
    // TODO : Implementing heartbeat + ping
    // TODO : Stress test 1
    //receive_bidirectional_stream(connection).await;
    connection.send_datagram(bytes::Bytes::from("Hello from client"))?;
    while let Ok(received_bytes) = connection.read_datagram().await {
        // Because it is a unidirectional stream, we can only receive not send back.
        println!(
            "Unreliable message received: {}",
            String::from_utf8(received_bytes.to_vec())?
        );
    }

    Ok(())
}

async fn open_bidirectional_stream(connection: Connection) -> Result<(), Box<dyn Error>> {
    let (mut send, mut recv) = connection.open_bi().await?;

    send.write_all(b"test").await?;
    send.finish().await?;

    let received = recv.read_to_end(PAYLOAD_SIZE).await?;
    info!("Message received : {}", String::from_utf8(received)?);

    Ok(())
}

async fn receive_bidirectional_stream(connection: Connection) -> Result<(), Box<dyn Error>> {
    while let Ok((mut send, mut recv)) = connection.accept_bi().await {
        // Because it is a bidirectional stream, we can both send and receive.
        info!(
            "Message received: {}",
            String::from_utf8(recv.read_to_end(PAYLOAD_SIZE).await?)?
        );

        send.write_all(b"response").await?;
        info!("Sent message \"response\" to the server.");
        send.finish().await?;
        info!("Bidirectional stream successfully closed.");
    }

    Ok(())
}

async fn open_unidirectional_stream(connection: Connection) -> Result<(), Box<dyn Error>> {
    let mut send = connection.open_uni().await?;

    send.write_all(b"test").await?;
    send.finish().await?;

    Ok(())
}

async fn receive_unidirectional_stream(connection: Connection) -> Result<(), Box<dyn Error>> {
    while let Ok(mut recv) = connection.accept_uni().await {
        // Because it is a unidirectional stream, we can only receive not send back.
        println!("{:?}", recv.read_to_end(50).await?);
    }

    Ok(())
}

// Implementation of `ServerCertVerifier` that verifies everything as trustworthy.
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl rustls::client::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &rustls::ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::ServerCertVerified::assertion())
    }
}
