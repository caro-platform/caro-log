use chrono::{DateTime, Local};
use log::Level;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogMessage {
    pub timestamp: DateTime<Local>,
    pub level: Level,
    pub target: String,
    pub message: String,
}

impl LogMessage {
    pub fn new(level: Level, target: String, message: String) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            target,
            message,
        }
    }
}
