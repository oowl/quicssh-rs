mod client;
mod server;

use log4rs::append::console::{ConsoleAppender, Target};
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
    /// Log level, Default Error
    #[clap(long)]
    log_level: Option<LevelFilter>,
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

    let level = match args.log_level {
        Some(log_level) => log_level,
        None => LevelFilter::Error,
    };
    let config = match args.log_file {
        Some(log_file) => {
            let logfile = FileAppender::builder()
                .encoder(Box::<PatternEncoder>::default())
                .build(log_file)
                .unwrap();

            Config::builder()
                .appender(Appender::builder().build("logfile", Box::new(logfile)))
                .build(Root::builder().appender("logfile").build(level))
                .unwrap()
        }
        None => {
            let stderr = ConsoleAppender::builder()
                .encoder(Box::<PatternEncoder>::default())
                .target(Target::Stderr)
                .build();
            Config::builder()
                .appender(Appender::builder().build("stderr", Box::new(stderr)))
                .build(Root::builder().appender("stderr").build(level))
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
