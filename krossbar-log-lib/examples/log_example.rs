use log::*;

use krossbar_log_lib::init_logger;

#[tokio::main]
async fn main() {
    init_logger("com.examples.logging", LevelFilter::Trace, false).await;

    error!("Error message");
    warn!("Warning message");
    info!("Info message");
    debug!("Debug message");
    trace!("Trace message");
}
