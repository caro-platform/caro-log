pub mod logger;

use krossbar_log_common::DEFAULT_LOGGER_SOCKET_PATH;
use log::LevelFilter;
pub use logger::Logger;

pub async fn init_logger(service_name: &str, level: LevelFilter, log_to_stdout: bool) {
    let logger = Box::new(
        Logger::new(
            service_name,
            level,
            log_to_stdout,
            Some(DEFAULT_LOGGER_SOCKET_PATH.into()),
        )
        .await
        .unwrap(),
    );

    log::set_boxed_logger(logger)
        .map(|()| log::set_max_level(level))
        .unwrap();
}
