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
