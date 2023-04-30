// #![cfg(feature = "rustls")]

use quinn::{ClientConfig, Endpoint, ServerConfig};
use std::{error::Error, net::SocketAddr, sync::Arc};
use std::{
    path::PathBuf,
    net::ToSocketAddrs,
};
use clap::Parser;
use url::Url;
use tokio::io::{BufReader, AsyncReadExt};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};

#[derive(Parser, Debug)]
#[clap(name = "client")]
pub struct Opt {
    /// file to log TLS keys to for debugging
    #[clap(long = "keylog")]
    keylog: bool,
    /// Enable stateless retries
    #[clap(long = "stateless-retry")]
    stateless_retry: bool,
    /// Sewrver address
    url: Url,
}

/// Enables MTUD if supported by the operating system
#[cfg(not(any(windows, os = "linux")))]
pub fn enable_mtud_if_supported(_client_config: &mut ClientConfig) {}

/// Enables MTUD if supported by the operating system
#[cfg(any(windows, os = "linux"))]
pub fn enable_mtud_if_supported(client_config: &mut ClientConfig) {
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.mtu_discovery_config(Some(quinn::MtuDiscoveryConfig::default()));
    client_config.transport_config(Arc::new(transport_config));
}

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

fn configure_client() -> Result<ClientConfig, Box<dyn Error>> {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

        let mut client_config = ClientConfig::new(Arc::new(crypto));
        enable_mtud_if_supported(&mut client_config);

    Ok(client_config)
}

/// Constructs a QUIC endpoint configured for use a client only.
///
/// ## Args
///
/// - server_certs: list of trusted certificates.
#[allow(unused)]
pub fn make_client_endpoint(
    bind_addr: SocketAddr,
) -> Result<Endpoint, Box<dyn Error>> {
    let client_cfg = configure_client()?;
    let mut endpoint = Endpoint::client(bind_addr)?;
    endpoint.set_default_client_config(client_cfg);
    Ok(endpoint)
}


#[tokio::main]
pub async fn run(options: Opt) -> Result<(), Box<dyn Error>> {
    let url = options.url;
    if url.scheme() != "quic" {
        return Err("URL scheme must be quic".into());
    }

    let remote = (url.host_str().unwrap(), url.port().unwrap_or(4433))
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| "couldn't resolve to an address")?;

    println!("Connecting to {:?}", remote);

    let endpoint =  make_client_endpoint("0.0.0.0:0".parse().unwrap())?;
    // connect to server
    let connection = endpoint
        .connect(remote, url.host_str().unwrap_or("localhost"))
        .unwrap()
        .await
        .unwrap();
    println!("[client] connected: addr={}", connection.remote_address());


    let (mut send, mut recv) = connection
        .open_bi()
        .await
        .map_err(|e| format!("failed to open stream: {}", e))?;

    let recv_thread = async move {
        let mut buf = vec![0; 2048];
        let mut stdout = tokio::io::stdout();

        loop {
            match recv.read(&mut buf).await {
                // Return value of `Ok(0)` signifies that the remote has
                // closed
                Ok(None) => {
                    // println!("error recv data from server");
                    continue;
                },
                Ok(Some( n)) => {
                    // println!("recv data from server {} bytes", n);
                    // Copy the data back to socket
                    match stdout.write_all(&buf[..n]).await {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error writing to stdout stream: {}", e);
                            return;
                        }
                    }
                }
                Err(err) => {
                    // Unexpected socket error. There isn't much we can do
                    // here so just stop processing.
                    println!("error recv data from quic {}", err);
                    return;
                }
            }
        }
    };

    let write_thread = async move {
        let mut buf =  [0; 2048];
        let mut stdin = tokio::io::stdin();

        loop {
            match stdin.read(&mut buf).await {
                // Return value of `Ok(0)` signifies that the remote has
                // closed
                Ok(n) => {
                    if n == 0 {
                        // println!("error recv data from stdin");
                        continue;
                    }
                    // println!("send data to server {} bytes", n);
                    // Copy the data back to socket
                    if send.write_all(&buf[..n]).await.is_err() {
                        // Unexpected socket error. There isn't much we can
                        // do here so just stop processing.
                        // println!("error send data to server");
                        return;
                    }
                }
                Err(err) => {
                    // Unexpected socket error. There isn't much we can do
                    // here so just stop processing.
                    // println!("error recv data from ssh {}", err);
                    return;
                }
            }
        }
    };

    tokio::join!(recv_thread, write_thread);
    Ok(())
}