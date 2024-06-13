use krossbar_log_common::log_message::LogMessage;

pub mod args;
mod client;
pub mod logger;
pub mod rotator;
mod writer;

pub struct LogEvent {
    pub pid: i32,
    pub service_name: String,
    pub message: LogMessage,
}
