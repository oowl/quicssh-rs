// #![cfg(feature = "rustls")]

use clap::Parser;
use quinn::{ClientConfig, Endpoint};
use std::{error::Error, net::SocketAddr, sync::Arc, net::ToSocketAddrs};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::Url;

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn, Level};


#[derive(Parser, Debug)]
#[clap(name = "client")]
pub struct Opt {
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
pub fn make_client_endpoint(bind_addr: SocketAddr) -> Result<Endpoint, Box<dyn Error>> {
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

    info!("[client] Connecting to {:?}", remote);

    let endpoint = make_client_endpoint("0.0.0.0:0".parse().unwrap())?;
    // connect to server
    let connection = endpoint
        .connect(remote, url.host_str().unwrap_or("localhost"))
        .unwrap()
        .await
        .unwrap();
    info!("[client] connected: addr={}", connection.remote_address());

    let (mut send, mut recv) = connection
        .open_bi()
        .await
        .map_err(|e| format!("failed to open stream: {}", e))?;

    let recv_thread = async move {
        let mut buf = vec![0; 2048];
        let mut writer = tokio::io::BufWriter::with_capacity(1, tokio::io::stdout());

        loop {
            match recv.read(&mut buf).await {
                // Return value of `Ok(0)` signifies that the remote has
                // closed
                Ok(None) => {
                    continue;
                }
                Ok(Some(n)) => {
                    debug!("[client] recv data from quic server {} bytes", n);
                    // Copy the data back to socket
                    match writer.write_all(&buf[..n]).await {
                        Ok(_) => (),
                        Err(e) => {
                            error!("[client] write to stdout error: {}", e);
                            return;
                        }
                    }
                }
                Err(err) => {
                    // Unexpected socket error. There isn't much we can do
                    // here so just stop processing.
                    error!("[client] recv data from quic server error: {}", err);
                    return;
                }
            }
            if writer.flush().await.is_err() {
                error!("[client] recv data flush stdout error");
            }
        }
    };

    let write_thread = async move {
        let mut buf = [0; 2048];
        let mut reader = tokio::io::BufReader::with_capacity(1, tokio::io::stdin());

        loop {
            match reader.read(&mut buf).await {
                // Return value of `Ok(0)` signifies that the remote has
                // closed
                Ok(n) => {
                    if n == 0 {
                        continue;
                    }
                    debug!("[client] recv data from stdin {} bytes", n);
                    // Copy the data back to socket
                    if send.write_all(&buf[..n]).await.is_err() {
                        // Unexpected socket error. There isn't much we can
                        // do here so just stop processing.
                        info!("[client] send data to quic server error");
                        return;
                    }
                }
                Err(err) => {
                    // Unexpected socket error. There isn't much we can do
                    // here so just stop processing.
                    info!("[client] recv data from stdin error: {}", err);
                    return;
                }
            }
        }
    };

    tokio::select! {
        _ = recv_thread => (),
        _ = write_thread => (),
    }

    info!("[client] exit client");

    Ok(())
}
