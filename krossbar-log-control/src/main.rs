use std::path::PathBuf;

use clap::{self, Parser, Subcommand};
use krossbar_bus_common::DEFAULT_HUB_SOCKET_PATH;
use krossbar_bus_lib::Service;

use krossbar_log_common::{
    logger_interface::{
        SetLogLevel, LOGGER_SERVICE_NAME, LOG_CLIENTS_METHOD_NAME, SET_LOG_LEVEL_METHOD_NAME,
    },
    LOG_CONTROL_SERVICE_NAME,
};

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    /// List connected services
    List,
    SetLogLevel {
        /// Log files location
        #[clap(short, long, value_parser)]
        service_name: String,
        /// Log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE
        #[clap(short, long, value_parser)]
        level: log::LevelFilter,
    },
}

/// Krossbar log control
#[derive(Parser, Debug, Clone)]
#[clap(version, about, long_about = None)]
struct Args {
    /// Log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE
    #[clap(short, long, value_parser)]
    pub log_level: log::LevelFilter,

    /// Command
    #[clap(subcommand)]
    pub command: Commands,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut bus = Service::new(
        LOG_CONTROL_SERVICE_NAME,
        &PathBuf::from(DEFAULT_HUB_SOCKET_PATH),
    )
    .await
    .expect("Failed to register logging service");

    let client = bus.connect(LOGGER_SERVICE_NAME).await.unwrap();

    match args.command {
        Commands::List => {
            let clients: Vec<String> = client.get(LOG_CLIENTS_METHOD_NAME).await.unwrap();
            println!("Logger clients: {clients:?}");
        }
        Commands::SetLogLevel {
            service_name,
            level,
        } => {
            client
                .message(
                    SET_LOG_LEVEL_METHOD_NAME,
                    &SetLogLevel {
                        service_name: service_name.clone(),
                        level,
                    },
                )
                .await
                .unwrap();

            println!("Succesfully changed log {service_name} log level to {level}");
        }
    }

    Ok(())
}
