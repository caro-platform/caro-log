use tokio::sync::mpsc::Sender;

use karo_log_common::LOGGING_SERVICE_NAME;

use karo_bus_lib::{simple_peer::SimplePeer, Bus};

pub struct LoggerClient {
    peer_connection_tx: Sender<(String, SimplePeer)>,
}

impl LoggerClient {
    pub fn new(tx: Sender<(String, SimplePeer)>) -> Self {
        Self {
            peer_connection_tx: tx,
        }
    }

    pub async fn connect(self, bus: &mut Bus) -> crate::Result<()> {
        let peer = bus.log_connect(LOGGING_SERVICE_NAME).await?;

        self.peer_connection_tx
            .send((bus.service_name(), peer))
            .await
            .unwrap();
        Ok(())
    }
}
