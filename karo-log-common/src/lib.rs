pub mod log_message;

pub const LOG_CONTROL_SERVICE_NAME: &str = "karo.log.control";
pub const SET_LOG_LEVEL_METHOD_NAME: &str = "!set_log_level";

pub const LOGGING_SERVICE_NAME: &str = "karo.logger";
pub const LOGGING_METHOD_NAME: &str = "log";
pub const DEFAULT_LOG_LOCATION: &str = "/var/log/karo/karo.log";

pub const ROTATED_LOG_TIMESTAMP_FORMAT: &str = "%Y_%m_%d_%H_%M_%S";
