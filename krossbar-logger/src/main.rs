//! Krossbar logging service
//!
//! # Usage:
//!
//! ```bash
//! Usage: krossbar-logger [OPTIONS]
//!
//! Options:
//! -l, --log-level <LOG_LEVEL>
//!         Logger self log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE [default: DEBUG]
//!     --log-location <LOG_LOCATION>
//!         Log file location [default: /var/log/krossbar/krossbar.log]
//! -n, --num-bytes-rotate <NUM_BYTES_ROTATE>
//!         Max log file size in bytes [default: 1000000]
//! -k, --keep-num-files <KEEP_NUM_FILES>
//!         How many rotated log files to keep [default: 10]
//! -h, --help
//!         Print help
//! -V, --version
//!         Print version
//! ```
//!

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
