mod server;
mod client;

use std::{
    ascii, fs, io,
    net::SocketAddr,
    path::{self, Path, PathBuf},
    str,
    sync::Arc,
};
use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Server
    Server(server::Opt),
    /// Client
    Client(client::Opt)
}


fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Server(server) => {
            println!("Server: {:#?}", server);
            let err = server::run(server);
            match err {
                Ok(_) => {},
                Err(e) => {
                    println!("Error: {:#?}", e);
                }
            }
        },
        Commands::Client(client) => {
            println!("Client: {:#?}", client);
            let err = client::run(client);
            match err {
                Ok(_) => {},
                Err(e) => {
                    println!("Error: {:#?}", e);
                }
            }
        }
    };
}