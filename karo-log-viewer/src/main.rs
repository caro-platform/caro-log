pub mod file_trait;
pub mod rotated_file;

use clap::{self, Parser};
use log::LevelFilter;

use karo_log_common::DEFAULT_LOG_LOCATION;

use karo_log_lib::Logger as LibLogger;

/// Karo log viewer
#[derive(Parser, Debug, Clone)]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// Log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE
    #[clap(short, long, value_parser, default_value_t = LevelFilter::Info)]
    pub log_level: log::LevelFilter,

    /// Log files location
    #[clap(long, value_parser, default_value_t = DEFAULT_LOG_LOCATION.into())]
    pub log_location: String,
}

fn main() {
    let args = Args::parse();

    let _ = LibLogger::new(args.log_level, true);
}
