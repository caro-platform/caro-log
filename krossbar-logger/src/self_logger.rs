use std::sync::Mutex;

use chrono::Local;
use futures::{channel::mpsc::Sender, executor::block_on, SinkExt};
use log::{LevelFilter, Metadata, Record};
use tokio::net::unix;

use krossbar_log_common::{log_message::LogMessage, logger_interface::LOGGER_SERVICE_NAME};

use crate::LogEvent;

pub struct SelfLogger {
    pid: unix::pid_t,
    service_name: String,
    level: LevelFilter,
    log_sender: Mutex<Sender<LogEvent>>,
}

impl SelfLogger {
    pub fn new(level: LevelFilter, log_sender: Sender<LogEvent>) -> Self {
        Self {
            pid: std::process::id() as unix::pid_t,
            service_name: LOGGER_SERVICE_NAME.into(),
            level,
            log_sender: Mutex::new(log_sender),
        }
    }
}

impl log::Log for SelfLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let _ = block_on(self.log_sender.lock().unwrap().send(LogEvent {
                pid: self.pid,
                service_name: self.service_name.clone(),
                message: LogMessage {
                    level: record.level(),
                    target: record.metadata().target().into(),
                    timestamp: Local::now(),
                    message: format!("{}", record.args()),
                },
            }));
        }
    }

    fn flush(&self) {}
}
