use std::{
    io::{self, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};

use chrono::Local;
use colored::Colorize;
use futures::{select, FutureExt};
use log::{Level, LevelFilter, Log, Record};
use tokio::{
    net::UnixStream,
    runtime::Handle,
    sync::mpsc::{channel, Receiver, Sender},
};

use krossbar_log_common::{log_message::LogMessage, REGISTER_METHOD_NAME};
use krossbar_rpc::{Error, Result, RpcData};

use crate::rpc::Rpc;

/// How often the library tries to reconnect to a logger
const RECONNECT_PERIOD: Duration = Duration::from_millis(1000);
/// How many message to store in a buffer
const LOG_BUFFER_SIZE: usize = 100;

/// Logger handle to use for running the logger
pub struct Logger {
    /// Client service name
    service_name: String,
    /// Logger RPC handle
    rpc: Option<Rpc>,
    /// Last sussessfull logger connection
    last_connect_ts_ms: SystemTime,
    /// Logger socket path
    logger_socket_path: Option<PathBuf>,
    /// Receiving part of log messages channel
    log_receiver: Receiver<LogMessage>,
    /// Logging level
    _level: Arc<AtomicUsize>,
}

/// Global [Log] handle
struct LogHandle {
    /// If log to stdout
    log_to_stdout: bool,
    /// If send messages to the logger
    log_to_rpc: bool,
    /// Logging level
    level: Arc<AtomicUsize>,
    /// Sending part of the log messages channel
    log_sender: Sender<LogMessage>,
}

impl Logger {
    /// Create Logging handle, which can be used to run logger message sending.
    /// **service_name** is a client service name. It must be uniques across the system.
    /// **log_to_stdout** sets if logger should log to stdout. If set, library
    /// logs to stdout even if it then sends messages to the logger.
    /// **logger_socket_path** sets logger path. If is some, logging lib tries to connect
    /// to the logger at the provided path.
    pub async fn new(
        service_name: &str,
        level: LevelFilter,
        log_to_stdout: bool,
        logger_socket_path: Option<PathBuf>,
    ) -> Result<Logger> {
        let log_to_rpc = logger_socket_path.is_some();

        let rpc = if logger_socket_path.is_none() {
            None
        } else {
            Some(Self::connect(&service_name, logger_socket_path.clone().unwrap()).await?)
        };

        let (log_sender, log_receiver) = channel(LOG_BUFFER_SIZE);
        let arc_level = Arc::new(AtomicUsize::new(level as usize));

        let this = Self {
            service_name: service_name.into(),
            _level: arc_level.clone(),
            rpc,
            last_connect_ts_ms: SystemTime::now(),
            logger_socket_path: logger_socket_path,
            log_receiver,
        };

        let log_handle = Box::new(LogHandle::new(
            log_to_stdout,
            log_to_rpc,
            arc_level,
            log_sender,
        ));

        log::set_boxed_logger(log_handle)
            .map(|()| log::set_max_level(level))
            .unwrap();

        Ok(this)
    }

    pub async fn connect(service_name: &str, socket_path: PathBuf) -> Result<Rpc> {
        let socket = UnixStream::connect(socket_path)
            .await
            .map_err(|_| Error::PeerDisconnected)?;

        let mut rpc = Rpc::new(socket);
        let call = rpc
            .call(REGISTER_METHOD_NAME, &service_name.to_owned())
            .await?;

        match call.data {
            RpcData::Response(res) => {
                res?;
            }
            m => {
                return Err(Error::InternalError(format!(
                    "Invalid response on connect from logger: {m:?}"
                )));
            }
        }

        Ok(rpc)
    }

    /// Run logger message sending. Can be ommited if set to log only to stdout.
    pub async fn run(mut self) {
        loop {
            select! {
                message = self.log_receiver.recv().fuse() => {
                    if let Some(message) = message {
                        self.send_rpc_message(&message).await
                    } else {
                        eprintln!("Log handle closed");
                        break;
                    }
                }
                incoming = self.rpc.as_mut().unwrap().read_message().fuse() => {
                    eprintln!("Incoming command: {incoming:?}")
                }
            };
        }
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

    async fn send_rpc_message(&mut self, log_message: &LogMessage) {
        let internal_log_message = |message: String| -> LogMessage {
            LogMessage {
                timestamp: Local::now(),
                level: Level::Info,
                target: "logger".to_owned(),
                message: message,
            }
        };

        let rpc = self.rpc.as_mut().unwrap();

        // Failed to send message to logger. Check if we already want to reconnect
        if rpc.send_log(&log_message).await.is_err() {
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
                    self.last_connect_ts_ms = SystemTime::now();

                    rpc.replace_stream(new_rpc);

                    let _ = rpc.send_log(&log_message).await;
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

impl LogHandle {
    pub fn new(
        log_to_stdout: bool,
        log_to_rpc: bool,
        level: Arc<AtomicUsize>,
        log_sender: Sender<LogMessage>,
    ) -> Self {
        Self {
            log_to_stdout,
            level,
            log_sender,
            log_to_rpc,
        }
    }
}

impl Log for LogHandle {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() as usize <= self.level.load(Ordering::Relaxed)
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
                Logger::log_to_stdout(&log_message)
            }

            if self.log_to_rpc {
                // If we're inside Tokio runtime, we spawn a task. Otherwise we'll block to send
                if let Ok(handle) = Handle::try_current() {
                    let sender = self.log_sender.clone();

                    handle.spawn(async move {
                        if sender.send(log_message).await.is_err() {
                            eprintln!("Failed to send log message into channel");
                        }
                    });
                } else {
                    if self.log_sender.blocking_send(log_message).is_err() {
                        eprintln!("Failed to send log message into channel");
                    }
                }
            }
        }
    }

    fn flush(&self) {
        if self.log_to_stdout {
            let _ = io::stdout().flush();
        }
    }
}
