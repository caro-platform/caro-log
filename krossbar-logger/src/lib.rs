use krossbar_log_common::log_message::LogMessage;
use tokio::net::unix;

pub mod args;
mod client;
pub mod logger;
pub mod rotator;
#[cfg(not(feature = "disable-self-logger"))]
mod self_logger;
mod writer;

pub struct LogEvent {
    pub pid: unix::pid_t,
    pub service_name: String,
    pub message: LogMessage,
}
