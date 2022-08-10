use crate::args;

use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};

use karo_bus_lib::signal::Signal;
use log::*;

use karo_log_common::log_message::LogMessage;

use crate::rotator::Rotator;

#[derive(Clone)]
pub struct Logger {
    log_file: Arc<Mutex<Option<File>>>,
    rotated_sigal: Arc<Signal<String>>,
    args: args::Args,
    current_file_num_bytes: Arc<AtomicU64>,
}

impl Logger {
    pub fn new(args: &args::Args, rotated_signal: Signal<String>) -> Self {
        let mut this = Self {
            log_file: Arc::new(Mutex::new(None)),
            args: args.clone(),
            rotated_sigal: Arc::new(rotated_signal),
            current_file_num_bytes: Arc::new(AtomicU64::new(0)),
        };

        this.open_log_file();
        this
    }

    fn open_log_file(&mut self) {
        *self.log_file.lock().unwrap() = Some(
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(&self.args.log_location)
                .unwrap(),
        );
    }

    fn close_log_file(&mut self) {
        *self.log_file.lock().unwrap() = None;
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

        // New current log len
        self.current_file_num_bytes
            .fetch_add(log_line.len() as u64, Ordering::Relaxed);

        debug!("{}", log_line);

        match *self.log_file.lock().unwrap() {
            Some(ref mut log_file) => {
                if let Err(err) = log_file.write(log_line.as_bytes()) {
                    eprintln!("Failed to write log message: {}", err.to_string())
                }
            }
            _ => {
                eprintln!("Failed to write log message. Log file is closed");
            }
        }

        self.check_rotate()
    }

    fn check_rotate(&mut self) {
        if self.current_file_num_bytes.load(Ordering::Relaxed) < self.args.num_bytes_rotate {
            return;
        }

        self.close_log_file();

        let rotate_file_path = Rotator::rotate(&self.args);

        self.open_log_file();

        self.rotated_sigal.emit(rotate_file_path);
    }
}
