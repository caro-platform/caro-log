use chrono::{DateTime, Local};
use log::Level;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LogMessage {
    pub timestamp: DateTime<Local>,
    pub service_name: String,
    pub pid: u32,
    pub level: Level,
    pub target: String,
    pub message: String,
}

impl LogMessage {
    pub fn new(pid: u32, level: Level, target: String, message: String) -> Self {
        Self {
            timestamp: Local::now(),
            service_name: "".into(),
            pid,
            level,
            target,
            message,
        }
    }
}
