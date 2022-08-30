use clap::{self, Parser};
use log::*;

use karo_bus_lib::Bus;

use karo_log_common::{LOG_CONTROL_SERVICE_NAME, SET_LOG_LEVEL_METHOD_NAME};
use karo_log_lib::Logger as LibLogger;

/// Karo log control
#[derive(Parser, Debug, Clone)]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// Log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE
    #[clap(short, long, value_parser)]
    pub log_level: log::LevelFilter,

    /// Log files location
    #[clap(short, long, value_parser)]
    pub service_name: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let _ = LibLogger::new(LevelFilter::Debug, true);

    let mut bus = Bus::register(LOG_CONTROL_SERVICE_NAME)
        .await
        .expect("Failed to register logging service");

    let client = bus.connect(&args.service_name).await.unwrap();

    debug!(
        "Changing service '{}' log level to {}",
        args.service_name, args.log_level
    );

    client
        .call::<LevelFilter, ()>(SET_LOG_LEVEL_METHOD_NAME, &args.log_level)
        .await
        .unwrap();

    debug!("Succesfully set log level");

    Ok(())
}
