use std::{
    process,
    sync::{Arc, RwLock},
};

use colored::Colorize;
use log::{Level, LevelFilter, Log, Record};
use tokio::sync::mpsc::{self, Receiver, Sender};

use krossbar_bus_lib::simple_peer::SimplePeer;

use krossbar_log_common::{log_message::LogMessage, LOGGING_METHOD_NAME};

use crate::logger_client::LoggerClient;

pub struct Logger {
    pid: u32,
    level: Arc<RwLock<LevelFilter>>,
    tx: Sender<LogMessage>,
}

impl Logger {
    pub fn new(level: LevelFilter, log_to_stdout: bool) -> crate::Result<LoggerClient> {
        let (tx, rx) = mpsc::channel(64);
        let (peer_tx, peer_rx) = mpsc::channel(1);
        let (level_tx, level_rx) = mpsc::channel(1);

        let this = Self {
            pid: process::id(),
            level: Arc::new(RwLock::new(level)),
            tx,
        };

        this.start_sending_task(rx, peer_rx, level_rx, log_to_stdout);
        log::set_boxed_logger(Box::new(this)).map(|()| log::set_max_level(LevelFilter::Trace))?;

        Ok(LoggerClient::new(peer_tx, level_tx))
    }
}

impl Logger {
    fn start_sending_task(
        &self,
        mut rx: Receiver<LogMessage>,
        mut peer_rx: Receiver<(String, SimplePeer)>,
        mut level_rx: Receiver<LevelFilter>,
        log_to_stdout: bool,
    ) {
        let log_level = self.level.clone();

        tokio::spawn(async move {
            let mut service_name: String = "Unknown".into();
            let mut logger_connection: Option<SimplePeer> = None;
            let mut peer_tx_closed = false;

            loop {
                tokio::select! {
                    message = rx.recv() => {
                        if message.is_none() {
                            log::warn!(
                                "Failed to recieve mesage from a logging channel. Assuming shutting down"
                            );
                            return;
                        }

                        let mut message = message.unwrap();
                        message.service_name = service_name.clone();

                        if log_to_stdout || logger_connection.is_none() {
                            Self::log_to_stdout(&message);
                        }

                        if logger_connection.is_some() {
                            // The function will return an error if disconnected
                            // We'll log to stdout meanwhile and wait while reconneceted
                            if let Err(_) = logger_connection.as_ref()
                                .unwrap()
                                .call::<LogMessage, ()>(LOGGING_METHOD_NAME, &message)
                                .await
                            {
                                if !log_to_stdout {
                                    print!("Offline message: ");
                                    Self::log_to_stdout(&message);
                                }
                            }
                        }
                    },
                    Some(level) = level_rx.recv() => {
                        log::info!(
                            "Got new log level from the log control: {}", level
                        );

                        *log_level.write().unwrap() = level;
                    },
                    peer = peer_rx.recv(), if !peer_tx_closed => {
                        // None means user already set the peer connection, or just dropped connector handle.
                        // Both cases are valid and we just stop pooling it
                        if let Some((self_service_name, peer)) = peer {
                            service_name = self_service_name;
                            logger_connection = Some(peer)
                        } else {
                            peer_tx_closed = true;
                        }
                    }
                }
            }
        });
    }

    fn log_to_stdout(message: &LogMessage) {
        let colored_level = match message.level {
            Level::Error => "ERROR".bright_red(),
            Level::Warn => "WARNING".bright_yellow(),
            Level::Info => "INFO".bright_green(),
            Level::Debug => "DEBUG".bright_blue(),
            Level::Trace => "TRACE".bright_white(),
        };

        println!(
            "{}: {} > {}",
            colored_level,
            message.target.bright_white(),
            message.message
        );
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= *self.level.read().unwrap()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_message = LogMessage::new(
                self.pid,
                record.level(),
                format!("{}", record.target()),
                format!("{}", record.args()),
            );

            if let Err(_) = self.tx.try_send(log_message.clone()) {
                print!("Offline message: ");
                Self::log_to_stdout(&log_message);
            }
        }
    }

    fn flush(&self) {}
}
