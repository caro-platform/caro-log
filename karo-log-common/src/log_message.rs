use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LogMessage {
    pid: u32,
    message: String,
}

impl LogMessage {
    pub fn new(pid: u32, message: String) -> Self {
        Self { pid, message }
    }
}
