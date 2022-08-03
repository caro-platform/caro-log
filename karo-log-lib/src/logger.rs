use std::process;

use colored::Colorize;
use log::{Level, LevelFilter, Log, Record};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    oneshot::{self, Receiver as OneReceiver},
};

use karo_bus_lib::peer::Peer;

use karo_log_common::{log_message::LogMessage, LOGGING_METHOD_NAME};

use crate::logger_client::LoggerClient;

pub struct Logger {
    pid: u32,
    level: LevelFilter,
    tx: Sender<LogMessage>,
}

impl Logger {
    pub fn new(level: LevelFilter, log_to_stdout: bool) -> crate::Result<LoggerClient> {
        let (tx, rx) = mpsc::channel(64);
        let (peer_tx, peer_rx) = oneshot::channel();

        let this = Self {
            pid: process::id(),
            level,
            tx,
        };

        this.start_sending_task(rx, peer_rx, log_to_stdout);
        log::set_boxed_logger(Box::new(this)).map(|()| log::set_max_level(LevelFilter::Trace))?;

        Ok(LoggerClient::new(peer_tx))
    }
}

impl Logger {
    fn start_sending_task(
        &self,
        mut rx: Receiver<LogMessage>,
        mut peer_rx: OneReceiver<Peer>,
        log_to_stdout: bool,
    ) {
        tokio::spawn(async move {
            let mut logger_connection: Option<Peer> = None;

            loop {
                tokio::select! {
                    message = rx.recv() => {
                        if message.is_none() {
                            log::warn!(
                                "Failed to recieve mesage from a logging channel. Assuming shutting down"
                            );
                            return;
                        }

                        let message = message.unwrap();
                        if log_to_stdout || logger_connection.is_none() {
                            Self::log_to_stdout(&message);
                        }

                        if logger_connection.is_some() {
                            if let Err(err) = logger_connection.as_ref()
                                .unwrap()
                                .call::<LogMessage, ()>(LOGGING_METHOD_NAME, &message)
                                .await
                            {
                                eprintln!("Failed to send logging message: {}", err.to_string());
                            }
                        }
                    },
                    peer = &mut peer_rx => {
                        // If user just droms logger connector, we get an error here. Now we just ignore it
                        if let Ok(peer) = peer {
                            logger_connection = Some(peer)
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
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_message = LogMessage::new(
                self.pid,
                record.level(),
                format!("{}", record.target()),
                format!("{}", record.args()),
            );

            if let Err(err) = self.tx.try_send(log_message) {
                eprintln!(
                    "Failed to send logging message into channel: {}",
                    err.to_string()
                );
            }
        }
    }

    fn flush(&self) {}
}
