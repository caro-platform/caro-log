use clap::{self, Parser};
use log::LevelFilter;

use krossbar_log_common::DEFAULT_LOG_LOCATION;

/// Krossbar logger
#[derive(Parser, Debug, Clone)]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// Log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE
    #[clap(short, long, value_parser, default_value_t = LevelFilter::Info)]
    pub log_level: log::LevelFilter,

    /// Log file location
    #[clap(long, value_parser, default_value_t = DEFAULT_LOG_LOCATION.into())]
    pub log_location: String,

    /// Rotated file bytes
    #[clap(short, long, value_parser, default_value_t = 1_000_000)]
    pub num_bytes_rotate: u64,

    /// How mane rotated logs to keep
    #[clap(short, long, value_parser, default_value_t = 10)]
    pub keep_num_logs: usize,
}
