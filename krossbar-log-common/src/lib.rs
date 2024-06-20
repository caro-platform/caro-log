pub mod log_message;
pub mod logger_interface;

pub const LOG_CONTROL_SERVICE_NAME: &str = "krossbar.log.control";

pub const DEFAULT_LOG_LOCATION: &str = "/var/log/krossbar/krossbar.log";
pub const DEFAULT_LOGGER_SOCKET_PATH: &str = "/var/run/krossbar.logger.socket";

pub const ROTATED_LOG_TIMESTAMP_FORMAT: &str = "%Y_%m_%d_%H_%M_%S";
