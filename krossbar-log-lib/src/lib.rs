//! Krossbar logging library
//!
//! The library is used to connect to Krossbar logging service to send log messages
//! and receive log control commands.
//!
//! See [Krossbar log control](https://crates.io/crates/krossbar-log-control) documentation
//! on how to control logging.
//!
//! The library uses Unix stream connection to send logging messages, which means you need
//! running [Krossbar logger](https://crates.io/crates/krossbar-logger) to log mesage.
//! In case service can't connect to the logger, it logs to stdout.
//!
//! Also, you can use [Logger] manually to control whether log into stdout or send
//! message to the logger. Both option are independent.
//!
//! In case you use Krossbar logger, you have to run logging loop using [Logger::run].
//!
//! # Examples
//! ```rust
//! use std::time::Duration;
//!
//! use log::*;
//! use tokio::select;
//!
//! use krossbar_log_lib::init_logger;
//!
//! async fn log_example() {
//!     let logger = init_logger("com.examples.logging", LevelFilter::Trace, true).await;
//!
//!     tokio::spawn(logger.run());
//!
//!     loop {
//!         error!("Error message");
//!         warn!("Warning message");
//!         info!("Info message");
//!         debug!("Debug message");
//!         trace!("Trace message");
//!
//!         select! {
//!             _ = tokio::time::sleep(Duration::from_secs(1)) => {},
//!             _ = tokio::signal::ctrl_c() => { return; }
//!         }
//!     }
//! }
//! ```
pub mod logger;
mod rpc;

use log::LevelFilter;

use krossbar_log_common::DEFAULT_LOGGER_SOCKET_PATH;

pub use logger::Logger;

/// Init logger.
/// **service_name** is a client service name. It must be uniques across the system.
/// **log_to_stdout** sets if logger should log to stdout. If set, library
/// logs to stdout even if it then sends messages to the logger.
pub async fn init_logger(service_name: &str, level: LevelFilter, log_to_stdout: bool) -> Logger {
    Logger::new(
        service_name,
        level,
        log_to_stdout,
        Some(DEFAULT_LOGGER_SOCKET_PATH.into()),
    )
    .await
    .unwrap()
}
