use std::{path::PathBuf, process};

use colored::Colorize;
use krossbar_bus_common::protocols::hub::HUB_REGISTER_METHOD;
use log::{Level, LevelFilter, Log, Record};
use tokio::net::UnixStream;

use krossbar_common_rpc::{rpc::Rpc, Error, Result};
use krossbar_log_common::{
    log_message::LogMessage, DEFAULT_LOGGER_SOCKET_PATH, LOGGING_METHOD_NAME,
};

const RECONNECT_PERIO_MSECS: usize = 1000;

pub struct Logger {
    pid: u32,
    service_name: String,
    level: LevelFilter,
    rpc: Option<Rpc>,
    log_to_stdout: bool,
}

impl Logger {
    pub async fn new(
        service_name: String,
        level: LevelFilter,
        log_to_stdout: bool,
    ) -> Result<Logger> {
        let rpc = if log_to_stdout {
            None
        } else {
            Some(Self::connect(&service_name, DEFAULT_LOGGER_SOCKET_PATH.into()).await?)
        };

        let this = Self {
            pid: process::id(),
            service_name,
            level,
            rpc,
            log_to_stdout,
        };

        Ok(this)
    }

    pub async fn connect(service_name: &str, socket_path: PathBuf) -> Result<Rpc> {
        let socket = UnixStream::connect(socket_path)
            .await
            .map_err(|_| Error::PeerDisconnected)?;

        let rpc = Rpc::new(socket, "logger");
        let connection = rpc
            .call(HUB_REGISTER_METHOD, HubMessage::Register { service_name })
            .await?
            .await?;

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

            if let Err(_) = self.tx.try_send(log_message.clone()) {
                print!("Offline message: ");
                Self::log_to_stdout(&log_message);
            }
        }
    }

    fn flush(&self) {}
}
