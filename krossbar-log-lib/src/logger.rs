use std::{
    io::{self, Write},
    path::PathBuf,
    sync::Mutex,
    time::{Duration, SystemTime},
};

use chrono::Local;
use colored::Colorize;
use futures::{executor::block_on, select, FutureExt};
use log::{Level, LevelFilter, Log, Record};
use tokio::{net::UnixStream, runtime::Handle};

use krossbar_common_rpc::{rpc::Rpc, Error, Result};
use krossbar_log_common::{log_message::LogMessage, LOG_METHOD_NAME, REGISTER_METHOD_NAME};

const RECONNECT_PERIOD: Duration = Duration::from_millis(1000);

pub struct Logger {
    service_name: String,
    level: LevelFilter,
    rpc: Option<Mutex<Rpc>>,
    log_to_stdout: bool,
    last_connect_ts_ms: SystemTime,
    logger_socket_path: Option<PathBuf>,
}

impl Logger {
    pub async fn new(
        service_name: &str,
        level: LevelFilter,
        log_to_stdout: bool,
        logger_socket_path: Option<PathBuf>,
    ) -> Result<Logger> {
        let rpc = if logger_socket_path.is_none() {
            None
        } else {
            Some(Mutex::new(
                Self::connect(&service_name, logger_socket_path.clone().unwrap()).await?,
            ))
        };

        let this = Self {
            service_name: service_name.into(),
            level,
            rpc,
            log_to_stdout,
            last_connect_ts_ms: SystemTime::now(),
            logger_socket_path: logger_socket_path,
        };

        Ok(this)
    }

    pub async fn connect(service_name: &str, socket_path: PathBuf) -> Result<Rpc> {
        let socket = UnixStream::connect(socket_path)
            .await
            .map_err(|_| Error::PeerDisconnected)?;

        let mut rpc = Rpc::new(socket, "logger");
        let call = rpc
            .call::<String, ()>(REGISTER_METHOD_NAME, &service_name.to_owned())
            .await?;

        select! {
            call = call.fuse() => call?,
            _ = rpc.poll().fuse() => {}
        };

        Ok(rpc)
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

    async fn send_rpc_message(&self, log_message: &LogMessage) {
        let internal_log_message = |message: String| -> LogMessage {
            LogMessage {
                timestamp: Local::now(),
                level: Level::Info,
                target: "logger".to_owned(),
                message: message,
            }
        };

        let mut rpc = self.rpc.as_ref().unwrap().lock().unwrap();

        // Failed to send message to logger. Check if we already want to reconnect
        if rpc
            .send_message(LOG_METHOD_NAME, &log_message)
            .await
            .is_err()
        {
            // We want to reconnect
            if (SystemTime::now() - RECONNECT_PERIOD) > self.last_connect_ts_ms {
                Self::log_to_stdout(&internal_log_message(
                    "Logger is down. Trying to reconnect".into(),
                ));

                // Succesfully reconnected
                if let Ok(new_rpc) =
                    Self::connect(&self.service_name, self.logger_socket_path.clone().unwrap())
                        .await
                {
                    Self::log_to_stdout(&internal_log_message(
                        "Succesfully reconnected to a loger. Sending source message".into(),
                    ));

                    rpc.on_reconnected(new_rpc).await;

                    let _ = rpc.send_message(LOG_METHOD_NAME, &log_message).await;
                // Failed to reconnect
                } else {
                    Self::log_to_stdout(&internal_log_message(
                        "Failed to reconnect to a logger".into(),
                    ));

                    Self::log_to_stdout(&log_message)
                }
            // It's not time to reconnect. Log into stdout
            } else {
                Self::log_to_stdout(&log_message)
            }
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_message = LogMessage {
                timestamp: Local::now(),
                level: record.level(),
                target: record.metadata().target().to_owned(),
                message: format!("{}", record.args()),
            };

            if self.log_to_stdout {
                Self::log_to_stdout(&log_message)
            }

            if self.logger_socket_path.is_some() {
                // Block if we're inside a Tokio context
                if let Ok(handle) = Handle::try_current() {
                    handle.block_on(self.send_rpc_message(&log_message));
                // Block otherwise
                } else {
                    block_on(self.send_rpc_message(&log_message));
                }
            }
        }
    }

    fn flush(&self) {
        if self.log_to_stdout || self.rpc.is_none() {
            let _ = io::stdout().flush();
        }
    }
}
