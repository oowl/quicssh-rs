mod client;
mod server;

use env_logger::{Builder, Env};

use clap::{Parser, Subcommand};
use log::error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    // we should still hold this struct for future extension
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Server
    Server(server::Opt),
    /// Client
    Client(client::Opt),
}

fn main() {
    let args = Cli::parse();

    Builder::from_env(Env::default().default_filter_or("info")).init();

    match args.command {
        Commands::Server(server) => {
            let err = server::run(server);
            match err {
                Ok(_) => {}
                Err(e) => {
                    error!("Error: {:#?}", e);
                }
            }
        }
        Commands::Client(client) => {
            let err = client::run(client);
            match err {
                Ok(_) => {}
                Err(e) => {
                    error!("Error: {:#?}", e);
                }
            }
        }
    }
}
