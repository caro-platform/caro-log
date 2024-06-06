use log::{warn, LevelFilter};
use tokio::sync::mpsc::Sender;

use krossbar_log_common::{LOGGING_SERVICE_NAME, SET_LOG_LEVEL_METHOD_NAME};

use krossbar_bus_lib::{simple_peer::SimplePeer, Bus};

pub struct LoggerClient {
    peer_connection_tx: Sender<(String, SimplePeer)>,
    /// Sender to send log level changes
    level_tx: Sender<LevelFilter>,
}

impl LoggerClient {
    pub fn new(tx: Sender<(String, SimplePeer)>, level_tx: Sender<LevelFilter>) -> Self {
        Self {
            peer_connection_tx: tx,
            level_tx,
        }
    }

    pub async fn connect(self, bus: &mut Bus) -> crate::Result<()> {
        let peer = bus.simple_connect(LOGGING_SERVICE_NAME).await?;

        let level_tx = self.level_tx.clone();
        bus.register_method(SET_LOG_LEVEL_METHOD_NAME, move |level: LevelFilter| {
            let level_tx = level_tx.clone();

            async move {
                if let Err(err) = level_tx.send(level).await {
                    warn!("Failed to send level change message: {}", err.to_string())
                }

                ()
            }
        })?;

        self.peer_connection_tx
            .send((bus.service_name(), peer))
            .await
            .unwrap();
        Ok(())
    }
}
