use std::path::{Path, PathBuf};

use clap::{self, Parser, Subcommand};
use krossbar_bus_common::DEFAULT_HUB_SOCKET_PATH;
use krossbar_bus_lib::Service;
use log::*;

use krossbar_log_common::{
    LOGGER_SERVICE_NAME, LOG_CONTROL_SERVICE_NAME, SET_LOG_LEVEL_METHOD_NAME,
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
        log_level: log::LevelFilter,
    },
}

/// Krossbar log control
#[derive(Parser, Debug, Clone)]
#[clap(version, about, long_about = None)]
pub struct Args {
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

    // match args.command {
    //     Commands::List => {
    //         let clients: Vec<String> = client.call(LOG_CLIENTS_METHOD_NAME, params)
    //     }
    // }

    // debug!(
    //     "Changing service '{}' log level to {}",
    //     args.service_name, args.log_level
    // );

    // client
    //     .call::<LevelFilter, ()>(SET_LOG_LEVEL_METHOD_NAME, &args.log_level)
    //     .await
    //     .unwrap();

    // debug!("Succesfully set log level");

    Ok(())
}
