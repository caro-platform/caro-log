use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LogMessage {
    pub pid: u32,
    pub message: String,
}

impl LogMessage {
    pub fn new(pid: u32, message: String) -> Self {
        Self { pid, message }
    }
}
