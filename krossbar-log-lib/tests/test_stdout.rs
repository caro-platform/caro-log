use log::*;

use krossbar_log_lib::Logger;

// See test stdout output
#[tokio::test]
async fn test_stdout() {
    let logger = Box::new(
        Logger::new("krossbar.log.stdout", LevelFilter::Debug, true, None)
            .await
            .unwrap(),
    );

    log::set_boxed_logger(logger)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();

    error!("Error message");
    warn!("Warning message");
    info!("Info message");
    debug!("Debug message");
    trace!("Trace message");
}
