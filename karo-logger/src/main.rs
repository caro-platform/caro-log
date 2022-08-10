mod args;
mod logger;
mod rotator;

use clap::Parser;
use log::*;

use karo_bus_lib::Bus;

use karo_log_common::{log_message::LogMessage, LOGGING_METHOD_NAME, LOGGING_SERVICE_NAME};
use karo_log_lib::Logger as LibLogger;

use logger::Logger;

const LOG_ROTATED_SIGNAL: &str = "log_rotated";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Karo logging service");

    let args = args::Args::parse();

    let _ = LibLogger::new(args.log_level, true);

    let mut bus = Bus::register(LOGGING_SERVICE_NAME)
        .await
        .expect("Failed to register logging service");

    let rotated_signal = bus.register_signal::<String>(LOG_ROTATED_SIGNAL).unwrap();

    let logger = Logger::new(&args, rotated_signal);

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
