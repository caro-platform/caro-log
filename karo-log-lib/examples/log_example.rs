use log::*;

use karo_bus_lib::Bus;
use karo_log_lib::Logger;

#[tokio::main]
async fn main() {
    let log_connector = Logger::new(LevelFilter::Debug, false).unwrap();

    error!("Error message");
    warn!("Warning message");
    info!("Info message");
    debug!("Debug message");
    trace!("Trace message");

    let mut bus = Bus::register("com.examples.logging").await.unwrap();

    log_connector.connect(&mut bus).await.unwrap();

    error!("Error message");
    warn!("Warning message");
    info!("Info message");
    debug!("Debug message");
    trace!("Trace message");

    let _ = tokio::signal::ctrl_c().await;
}
