#![allow(unused)]

use std::error::Error;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use log::{error, info};
use pretty_logger::{Destination, Theme};
use quinn::crypto::rustls::QuicClientConfig;
use quinn::{ClientConfig, Connection, Endpoint};
use quinn::rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified};
use quinn::rustls::{DigitallySignedStruct, SignatureScheme};
use quinn::rustls::pki_types::{CertificateDer, ServerName, UnixTime};

static PAYLOAD_SIZE: usize = 1024;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    pretty_logger::init(
        Destination::Stdout,
        "info".parse().unwrap(),
        Theme::default(),
    )?;

    let mut crypto = quinn::rustls::ClientConfig::builder().dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    let mut client_config = ClientConfig::new(Arc::new(QuicClientConfig::try_from(crypto)?));

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
    send.finish()?;

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
        send.finish()?;
        info!("Bidirectional stream successfully closed.");
    }

    Ok(())
}

async fn open_unidirectional_stream(connection: Connection) -> Result<(), Box<dyn Error>> {
    let mut send = connection.open_uni().await?;

    send.write_all(b"test").await?;
    send.finish()?;

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
#[derive(Debug)]
struct SkipServerVerification;

impl SkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self)
    }
}

impl quinn::rustls::client::danger::ServerCertVerifier for SkipServerVerification {
    fn verify_server_cert(&self, end_entity: &CertificateDer<'_>, intermediates: &[CertificateDer<'_>], server_name: &ServerName<'_>, ocsp_response: &[u8], now: UnixTime) -> Result<ServerCertVerified, quinn::rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }
    
    fn verify_tls12_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, quinn::rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(&self, message: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, quinn::rustls::Error> {
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
