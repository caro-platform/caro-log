mod args;
mod client;
mod logger;
mod rotator;
mod self_logger;
mod writer;

use clap::Parser;
use log::*;

use krossbar_log_common::{log_message::LogMessage, DEFAULT_LOGGER_SOCKET_PATH};

use logger::Logger;
use tokio::net::unix;

pub struct LogEvent {
    pub pid: unix::pid_t,
    pub service_name: String,
    pub message: LogMessage,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Krossbar logging service");

    let args = args::Args::parse();

    // tokio::spawn(async move {
    let logger = Logger::new(args, DEFAULT_LOGGER_SOCKET_PATH.into());
    info!("Succesfully started logging service. Listening for messages");

    logger.run().await;

    info!("Shutting down");
    // });

    // let _ = tokio::signal::ctrl_c().await;

    Ok(())
}
