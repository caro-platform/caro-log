use tokio::sync::mpsc::Sender;

use karo_log_common::LOGGING_SERVICE_NAME;

use karo_bus_lib::{peer::Peer, Bus};

pub struct LoggerClient {
    peer_connection_tx: Sender<Peer>,
}

impl LoggerClient {
    pub fn new(tx: Sender<Peer>) -> Self {
        Self {
            peer_connection_tx: tx,
        }
    }

    pub async fn connect(self, bus: &mut Bus) -> crate::Result<()> {
        let peer = bus.connect(LOGGING_SERVICE_NAME).await?;

        self.peer_connection_tx.send(peer).await.unwrap();
        Ok(())
    }
}
