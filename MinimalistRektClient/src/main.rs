#![allow(unused)]

use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

#[macro_use]
extern crate pretty_env_logger;

use log::{error, info, log};
use quinn::crypto::rustls::QuicClientConfig;
use quinn::rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified};
use quinn::rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use quinn::rustls::{DigitallySignedStruct, SignatureScheme};
use quinn::{ClientConfig, Connection, Endpoint};
use tokio::io::AsyncWriteExt;

static PAYLOAD_SIZE: usize = 1024;

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
    open_bidirectional_stream(&connection).await?;

    Ok(())
}

async fn open_bidirectional_stream(connection: &Connection) -> Result<(), Box<dyn Error>> {
    let (mut send, mut recv) = connection.open_bi().await?;

    let receiver = async {
        loop {
            let mut bytes: [u8; 1500] = [0; 1500];

            let received = recv.read(&mut bytes).await?;
            match received {
                Some(received) => {
                    // Handle the received bytes
                    info!("Received {} bytes from server.", received);
                    info!(
                        "Received message: {:?}",
                        String::from_utf8(bytes[..received].to_vec())?
                    );
                }
                None => {
                    // The stream has been closed
                    info!("Server closed the bidirectional stream");
                    return Ok(());
                }
            }
        }
    };

    let sender = async {
        let mut flag = 1;
        while flag <= 15 {
            send.write_all(b"test000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").await?;
            send.write_all(b"test2").await?;
            send.write_all(b"test3").await?;
            send.flush().await?;

            flag += 1;
        }

        Ok::<(), Box<dyn Error>>(())
    };

    tokio::try_join!(receiver, sender)?;

    Ok(())
}

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
