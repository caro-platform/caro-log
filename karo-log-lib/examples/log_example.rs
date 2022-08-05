use std::time::Duration;

use log::*;

use karo_bus_lib::Bus;
use karo_log_lib::Logger;

#[tokio::main]
async fn main() {
    let log_connector = Logger::new(LevelFilter::Trace, false).unwrap();

    error!("Error message");
    warn!("Warning message");
    info!("Info message");
    debug!("Debug message");
    trace!("Trace message");

    let mut bus = Bus::register("com.examples.logging").await.unwrap();

    log_connector.connect(&mut bus).await.unwrap();

    loop {
        error!("Error message");
        warn!("Warning message");
        info!("Info message");
        debug!("Debug message");
        trace!("Trace message");

        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => continue,
            _ = tokio::signal::ctrl_c() => return
        };
    }
}
