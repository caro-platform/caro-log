use std::{
    io::{self, Write},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
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

use krossbar_common_rpc::{rpc::Rpc, Error, Result};
use krossbar_log_common::{log_message::LogMessage, LOG_METHOD_NAME, REGISTER_METHOD_NAME};

const RECONNECT_PERIOD: Duration = Duration::from_millis(1000);
const LOG_BUFFER_SIZE: usize = 100;

pub struct Logger {
    service_name: String,
    rpc: Option<Rpc>,
    last_connect_ts_ms: SystemTime,
    logger_socket_path: Option<PathBuf>,
    log_receiver: Receiver<LogMessage>,
    _level: Arc<AtomicUsize>,
    inside_log_context: Arc<AtomicBool>,
}

struct LogHandle {
    log_to_stdout: bool,
    log_to_rpc: bool,
    level: Arc<AtomicUsize>,
    log_sender: Sender<LogMessage>,
    inside_log_context: Arc<AtomicBool>,
}

impl Logger {
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
        let inside_log_context = Arc::new(AtomicBool::new(false));

        let this = Self {
            service_name: service_name.into(),
            _level: arc_level.clone(),
            rpc,
            last_connect_ts_ms: SystemTime::now(),
            logger_socket_path: logger_socket_path,
            log_receiver,
            inside_log_context: inside_log_context.clone(),
        };

        let log_handle = Box::new(LogHandle::new(
            log_to_stdout,
            log_to_rpc,
            arc_level,
            log_sender,
            inside_log_context,
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
                incoming = self.rpc.as_mut().unwrap().poll().fuse() => {
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

        self.inside_log_context.store(true, Ordering::Release);

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

        self.inside_log_context.store(false, Ordering::Release);
    }
}

impl LogHandle {
    pub fn new(
        log_to_stdout: bool,
        log_to_rpc: bool,
        level: Arc<AtomicUsize>,
        log_sender: Sender<LogMessage>,
        inside_log_context: Arc<AtomicBool>,
    ) -> Self {
        Self {
            log_to_stdout,
            level,
            log_sender,
            log_to_rpc,
            inside_log_context,
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

            let inside_log_context = self.inside_log_context.load(Ordering::Acquire);

            if self.log_to_stdout || inside_log_context {
                Logger::log_to_stdout(&log_message)
            }

            if self.log_to_rpc && !inside_log_context {
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
