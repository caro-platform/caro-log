use std::process;

use log::{LevelFilter, Log, Record};
use tokio::sync::mpsc::{self, Receiver, Sender};

use karo_bus_lib::{bus::Bus, peer::Peer};

use karo_log_common::{log_message::LogMessage, LOGGING_METHOD_NAME, LOGGING_SERVICE_NAME};

pub struct Logger {
    pid: u32,
    level: LevelFilter,
    tx: Sender<LogMessage>,
}

impl Logger {
    pub async fn init(bus: &mut Bus, level: LevelFilter) -> crate::Result<()> {
        let peer = bus.connect(LOGGING_SERVICE_NAME).await?;
        let (tx, rx) = mpsc::channel(64);

        let this = Box::new(Self {
            pid: process::id(),
            level,
            tx,
        });

        this.start_sending_task(rx, peer);
        log::set_boxed_logger(this).map(|()| log::set_max_level(LevelFilter::Trace))?;

        Ok(())
    }
}

impl Logger {
    fn start_sending_task(&self, mut rx: Receiver<LogMessage>, logging_service: Peer) {
        tokio::spawn(async move {
            loop {
                let message = rx.recv().await;

                if message.is_none() {
                    log::warn!(
                        "Failed to recieve mesage from a logging channel. Assuming shutting down"
                    );
                    return;
                }

                if let Err(err) = logging_service
                    .call::<LogMessage, ()>(LOGGING_METHOD_NAME, &message.unwrap())
                    .await
                {
                    eprintln!("Failed to send logging message: {}", err.to_string());
                }
            }
        });
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_message =
                LogMessage::new(self.pid, format!("{}: {}", record.level(), record.args()));

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
