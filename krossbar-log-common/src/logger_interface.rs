use log::LevelFilter;
use serde::{Deserialize, Serialize};

pub const LOGGER_SERVICE_NAME: &str = "krossbar.logger";

pub const SET_LOG_LEVEL_METHOD_NAME: &str = "set_log_level";
pub const LOG_CLIENTS_METHOD_NAME: &str = "clients";
pub const LOG_METHOD_NAME: &str = "log";
pub const REGISTER_METHOD_NAME: &str = "register";
pub const ROTATED_SIGNAL: &str = "rotated";

#[derive(Serialize, Deserialize)]
pub struct SetLogLevel {
    pub service_name: String,
    pub level: LevelFilter,
}
