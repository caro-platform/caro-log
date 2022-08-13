pub mod file_trait;
pub mod rotated_file;

use std::{path::PathBuf, time::Duration};

use clap::{self, Parser};
use file_trait::File;
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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let _ = LibLogger::new(args.log_level, true);

    let mut rotator =
        rotated_file::RotatedFile::new(PathBuf::from("/home/cosm/Downloads/tmp/karo.log"));

    loop {
        for _ in 0..12 {
            let lines = rotator.shift_and_read(file_trait::ShiftDirection::Down, 3);

            println!("-------------------->");
            for line in lines {
                println!("\\ {}", line);
            }
            println!("<--------------------");

            std::thread::sleep(Duration::from_millis(500));
        }

        for _ in 0..12 {
            let lines = rotator.shift_and_read(file_trait::ShiftDirection::Up, 3);

            println!("-------------------->");
            for line in lines {
                println!("/ {}", line);
            }
            println!("<--------------------");

            std::thread::sleep(Duration::from_millis(500));
        }
    }
}
