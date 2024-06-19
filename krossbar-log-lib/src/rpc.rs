use std::io::Cursor;

use bson::Document;
use serde::Serialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};

use krossbar_rpc::{Error, Result, RpcData, RpcMessage};

use krossbar_log_common::{log_message::LogMessage, LOG_METHOD_NAME};

pub struct Rpc {
    stream: UnixStream,
}

impl Rpc {
    pub fn new(stream: UnixStream) -> Self {
        Self { stream }
    }

    pub fn replace_stream(&mut self, rpc: Rpc) {
        self.stream = rpc.stream
    }

    pub async fn send_log(&mut self, message: &LogMessage) -> Result<()> {
        let data = bson::to_bson(message).map_err(|e| Error::ParamsTypeError(e.to_string()))?;

        let message = RpcMessage {
            id: -1,
            data: RpcData::Message {
                endpoint: LOG_METHOD_NAME.to_owned(),
                body: data,
            },
        };

        let doc = bson::to_document(&message).map_err(|e| Error::InternalError(e.to_string()))?;

        let mut buffer: Vec<u8> = Vec::new();
        doc.to_writer(&mut buffer)
            .map_err(|e| Error::InternalError(e.to_string()))?;

        self.stream
            .write_all(&buffer)
            .await
            .map_err(|_| Error::PeerDisconnected)
    }

    pub async fn call<T: Serialize>(&mut self, endpoint: &str, data: T) -> Result<RpcMessage> {
        let params = bson::to_bson(&data).map_err(|e| Error::ParamsTypeError(e.to_string()))?;

        let message = RpcMessage {
            id: -1,
            data: RpcData::Call {
                endpoint: endpoint.into(),
                params,
            },
        };

        let doc = bson::to_document(&message).map_err(|e| Error::InternalError(e.to_string()))?;

        let mut buffer: Vec<u8> = Vec::new();
        doc.to_writer(&mut buffer)
            .map_err(|e| Error::InternalError(e.to_string()))?;

        self.stream
            .write_all(&buffer)
            .await
            .map_err(|_| Error::PeerDisconnected)?;

        self.read_message().await
    }

    pub async fn read_message(&mut self) -> Result<RpcMessage> {
        // Read BSON len
        let mut len_buf = [0u8; 4];

        self.stream
            .read_exact(&mut len_buf)
            .await
            .map_err(|_| Error::PeerDisconnected)?;

        let len = i32::from_le_bytes(len_buf);

        // Read BSON body. Prepend BSON len to the rest of the data
        let mut data: Vec<u8> = len_buf.into();
        (&mut self.stream)
            .take((len - 4) as u64)
            .read_to_end(&mut data)
            .await
            .map_err(|_| Error::PeerDisconnected)?;

        let mut cursor = Cursor::new(data);
        let doc =
            Document::from_reader(&mut cursor).map_err(|e| Error::InternalError(e.to_string()))?;

        Ok(bson::from_document(doc).map_err(|e| Error::InternalError(e.to_string()))?)
    }
}
