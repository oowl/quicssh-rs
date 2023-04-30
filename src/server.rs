use std::{
    ascii, fs, io,
    net::SocketAddr,
    path::{self, Path, PathBuf},
    str,
    sync::Arc,
};
use std::error::Error;
use clap::{Args, Parser, Subcommand};
use quinn::{ClientConfig, Endpoint, ServerConfig};
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Parser, Debug)]
#[clap(name = "server")]
pub struct Opt {
    /// file to log TLS keys to for debugging
    #[clap(long = "keylog")]
    keylog: bool,
    /// TLS private key in PEM format
    #[clap(value_parser, short = 'k', long = "key", requires = "cert")]
    key: Option<PathBuf>,
    /// TLS certificate in PEM format
    #[clap(value_parser, short = 'c', long = "cert", requires = "key")]
    cert: Option<PathBuf>,
    /// Enable stateless retries
    #[clap(long = "stateless-retry")]
    stateless_retry: bool,
    /// Address to listen on
    #[clap(long = "listen", default_value = "0.0.0.0:4433")]
    listen: SocketAddr,
}

/// Returns default server configuration along with its certificate.
fn configure_server() -> Result<(ServerConfig, Vec<u8>), Box<dyn Error>> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
    let cert_der = cert.serialize_der().unwrap();
    let priv_key = cert.serialize_private_key_der();
    let priv_key = rustls::PrivateKey(priv_key);
    let cert_chain = vec![rustls::Certificate(cert_der.clone())];

    let mut server_config = ServerConfig::with_single_cert(cert_chain, priv_key)?;
    let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    transport_config.max_concurrent_uni_streams(0_u8.into());
    #[cfg(any(windows, os = "linux"))]
    transport_config.mtu_discovery_config(Some(quinn::MtuDiscoveryConfig::default()));

    Ok((server_config, cert_der))
}

#[allow(unused)]
pub fn make_server_endpoint(bind_addr: SocketAddr) -> Result<(Endpoint, Vec<u8>), Box<dyn Error>> {
    let (server_config, server_cert) = configure_server()?;
    let endpoint = Endpoint::server(server_config, bind_addr)?;
    Ok((endpoint, server_cert))
}

#[tokio::main]
pub async fn run(options: Opt) -> Result<(), Box<dyn Error>> {

    // let (certs, key) = if let (Some(key_path), Some(cert_path)) = (&options.key, &options.cert) {
    //     let key = fs::read(key_path).map_err(|err| format!("failed to read private key {}", err)).unwrap();
    //     let key = if key_path.extension().map_or(false, |x| x == "der") {
    //         rustls::PrivateKey(key)
    //     } else {
    //         let pkcs8: Vec<Vec<u8>> = rustls_pemfile::pkcs8_private_keys(&mut &*key)
    //             .map_err(|err| format!("malformed PKCS #8 private key {}", err).to_string()).unwrap();
    //         match pkcs8.into_iter().next() {
    //             Some(x) => rustls::PrivateKey(x),
    //             None => {
    //                 let rsa = rustls_pemfile::rsa_private_keys(&mut &*key)
    //                     .map_err(|err| format!("malformed PKCS #1 private key {}", err)).unwrap();
    //                 match rsa.into_iter().next() {
    //                     Some(x) => rustls::PrivateKey(x),
    //                     None => {
    //                         panic!("no private keys found")
    //                     }
    //                 }
    //             }
    //         }
    //     };
    //     let cert_chain = fs::read(cert_path).map_err(|err| format!("failed to read certificate chain {}", err)).unwrap();
    //     let cert_chain = if cert_path.extension().map_or(false, |x| x == "der") {
    //         vec![rustls::Certificate(cert_chain)]
    //     } else {
    //         rustls_pemfile::certs(&mut &*cert_chain).unwrap()
    //             .into_iter()
    //             .map(rustls::Certificate)
    //             .collect()
    //     };

    //     (cert_chain, key)
    // } else {
    //     panic!("no private keys found")
    // };

    let (endpoint, _) = make_server_endpoint(options.listen).unwrap();
    // accept a single connection
    loop {
        let incoming_conn = endpoint.accept().await.unwrap();
        let conn = incoming_conn.await.unwrap();
        println!(
            "[server] connection accepted: addr={}",
            conn.remote_address()
        );
        tokio::spawn(async move {
            handle_connection(conn).await;
        });
        // Dropping all handles associated with a connection implicitly closes it            
    }
}

async fn handle_connection(connection: quinn::Connection) {
    let ssh_stream = TcpStream::connect("127.0.0.1:22").await;
    let ssh_conn = match ssh_stream {
        Ok(conn) => conn,
        Err(e) => {
            println!("Error connecting to ssh server: {}", e);
            return;
        }
    };

    println!("ssh connection established\n");

    let (mut quinn_send, mut quinn_recv) = match connection.accept_bi().await {
        Ok(stream) => stream,
        Err(e) => {
            println!("Error opening quinn stream: {}", e);
            return;
        }
    };

    let (mut ssh_recv, mut ssh_write) = tokio::io::split(ssh_conn);


    let recv_thread= async move {
        let mut buf = [0; 2048];
        loop {
            match ssh_recv.read(&mut buf).await {
                Ok(n) => {
                    if n == 0 {
                        continue;
                    }
                    println!("recv data from ssh {} bytes", n);
                    match quinn_send.write_all(&buf[..n]).await {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error writing to quinn stream: {}", e);
                            return;
                        }
                    }
                }
                Err(e) => {
                    println!("Error reading from ssh stream: {}", e);
                    return;
                }
            }
        }
    };

    let write_thread = async move {
        let mut buf = [0; 2048];
        loop {
            match quinn_recv.read(&mut buf).await {
                Ok(None) => {
                    println!("quic connection closed");
                    continue;
                }
                Ok(Some(n)) => {
                    println!("{} bytes read from quic stream", n);
                    if n == 0 {
                        // println!("quic connection closed");
                        continue;
                    }
                    match ssh_write.write_all(&buf[..n]).await {
                        Ok(_) => (),
                        Err(e) => {
                            println!("Error writing to ssh stream: {}", e);
                            return;
                        }
                    }
                }
                Err(e) => {
                    println!("Error reading from quic stream: {}", e);
                    return;
                }
            }
        }
    };

    tokio::join!(recv_thread, write_thread);

}