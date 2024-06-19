use std::time::Duration;

use log::*;

use krossbar_log_lib::init_logger;
use tokio::select;

#[tokio::main]
async fn main() {
    let logger = init_logger("com.examples.logging", LevelFilter::Trace, true).await;

    tokio::spawn(logger.run());

    loop {
        error!("Error message");
        warn!("Warning message");
        info!("Info message");
        debug!("Debug message");
        trace!("Trace message");

        select! {
            _ = tokio::time::sleep(Duration::from_secs(1)) => {},
            _ = tokio::signal::ctrl_c() => { return; }
        }
    }
}
