mod client;
mod server;

use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use clap::{Parser, Subcommand};
use std::{
    str,
};

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
    Client(client::Opt),
}

fn main() {
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::default()))
        .build("log/output.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .unwrap();

    log4rs::init_config(config).unwrap();

    let args = Cli::parse();
    match args.command {
        Commands::Server(server) => {
            println!("Server: {:#?}", server);
            let err = server::run(server);
            match err {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {:#?}", e);
                }
            }
        }
        Commands::Client(client) => {
            println!("Client: {:#?}", client);
            let err = client::run(client);
            match err {
                Ok(_) => {}
                Err(e) => {
                    println!("Error: {:#?}", e);
                }
            }
        }
    };
}
