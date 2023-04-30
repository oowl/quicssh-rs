mod client;
mod server;

use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

use clap::{Parser, Subcommand};
use std::{
    str,
    path::{PathBuf},
};
use log::{error, LevelFilter};


#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Location of log, Defalt if 
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
    let mut config = Config::builder().build(Root::builder().build(LevelFilter::Info)).unwrap();

    if args.log_file.is_some() {
        let logfile = FileAppender::builder()
            .encoder(Box::new(PatternEncoder::default()))
            .build(args.log_file.unwrap())
            .unwrap();

        config = Config::builder()
            .appender(Appender::builder().build("logfile", Box::new(logfile)))
            .build(Root::builder().appender("logfile").build(LevelFilter::Info))
            .unwrap();
    }

    log4rs::init_config(config).unwrap();


    match args.command {
        Commands::Server(server) => {
            let err = server::run(server);
            match err {
                Ok(_) => {}
                Err(e) => {
                    error!("Error: {:#?}", e);
                    return;
                }
            }
        }
        Commands::Client(client) => {
            let err = client::run(client);
            match err {
                Ok(_) => {}
                Err(e) => {
                    error!("Error: {:#?}", e);
                    return;
                }
            }
        }
    };
}
