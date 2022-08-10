mod logger;

use clap::{self, Parser};
use log::*;

use karo_bus_lib::Bus;

use karo_log_common::{
    log_message::LogMessage, DEFAULT_LOG_LOCATION, LOGGING_METHOD_NAME, LOGGING_SERVICE_NAME,
};
use karo_log_lib::Logger as LibLogger;

use logger::Logger;

/// Karo bus monitor
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// Log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE
    #[clap(short, long, value_parser, default_value_t = LevelFilter::Info)]
    pub log_level: log::LevelFilter,

    /// Service to monitor
    #[clap(long, value_parser, default_value_t = DEFAULT_LOG_LOCATION.into())]
    pub log_location: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Karo logging service");

    let args = Args::parse();

    let _ = LibLogger::new(args.log_level, true);

    let logger = Logger::new(&args.log_location);

    let mut bus = Bus::register(LOGGING_SERVICE_NAME)
        .await
        .expect("Failed to register logging service");

    bus.register_method(LOGGING_METHOD_NAME, move |message: LogMessage| {
        let mut logger = logger.clone();
        async move {
            logger.log_message(message);
        }
    })
    .expect("Failed to register logging function");

    info!("Succesfully started logging service. Listening for messages");
    let _ = tokio::signal::ctrl_c().await;
    info!("Shutting down");

    Ok(())
}
