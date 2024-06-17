use krossbar_log_common::log_message::LogMessage;
use tokio::net::unix;

pub mod args;
mod client;
#[cfg(feature = "log-to-stdout")]
pub mod log_macros;
pub mod logger;
pub mod rotator;
#[cfg(not(feature = "log-to-stdout"))]
mod self_logger;
mod writer;

pub struct LogEvent {
    pub pid: unix::pid_t,
    pub service_name: String,
    pub message: LogMessage,
}
