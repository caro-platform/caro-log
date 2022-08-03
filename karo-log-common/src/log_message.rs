use log::Level;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct LogMessage {
    pub pid: u32,
    pub level: Level,
    pub target: String,
    pub message: String,
}

impl LogMessage {
    pub fn new(pid: u32, level: Level, target: String, message: String) -> Self {
        Self {
            pid,
            level,
            target,
            message,
        }
    }
}
