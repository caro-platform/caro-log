use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::{Arc, Mutex},
};

use log::*;

use karo_log_common::log_message::LogMessage;

#[derive(Clone)]
pub struct Logger {
    log_file: Arc<Mutex<File>>,
}

impl Logger {
    pub fn new(log_file_location: &str) -> Self {
        Self {
            log_file: Arc::new(Mutex::new(
                OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create(true)
                    .open(&log_file_location)
                    .unwrap(),
            )),
        }
    }

    pub fn log_message(&mut self, message: LogMessage) {
        let log_line = format!(
            "<{}> {}#{} [{}] {} > {}\n",
            message.timestamp.format("%d-%m-%Y %H:%M:%S%.3f"),
            message.service_name,
            message.pid,
            message.level,
            message.target,
            message.message
        );

        debug!("{}", log_line);

        if let Err(err) = self.log_file.lock().unwrap().write(log_line.as_bytes()) {
            eprintln!("Failed to write log message: {}", err.to_string())
        }
    }
}
