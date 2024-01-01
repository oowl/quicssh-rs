mod client;
mod server;

use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use clap::{Parser, Subcommand};
use log::{error, LevelFilter};
use std::{path::PathBuf, str};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Location of log, Default if
    #[clap(value_parser, long = "log")]
    log_file: Option<PathBuf>,
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

    let config = match args.log_file {
        Some(log_file) => {
            let logfile = FileAppender::builder()
                .encoder(Box::<PatternEncoder>::default())
                .build(log_file)
                .unwrap();

            Config::builder()
                .appender(Appender::builder().build("logfile", Box::new(logfile)))
                .build(Root::builder().appender("logfile").build(LevelFilter::Info))
                .unwrap()
        }
        None => {
            let stdout = ConsoleAppender::builder()
                .encoder(Box::<PatternEncoder>::default())
                .build();
            Config::builder()
                .appender(Appender::builder().build("stdout", Box::new(stdout)))
                .build(Root::builder().appender("stdout").build(LevelFilter::Info))
                .unwrap()
        }
    };

    log4rs::init_config(config).unwrap();

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
