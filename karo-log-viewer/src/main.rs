pub mod log_directory_entry;
pub mod log_directory_reader;
pub mod log_files;
pub mod screens;

use std::io::stdin;

use clap::{self, Parser};
use log::LevelFilter;

use karo_log_common::DEFAULT_LOG_LOCATION;
use termion::{
    event::{Event, Key},
    input::TermRead,
};

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
    let _args = Args::parse();

    let mut screens = screens::Screens::new();
    let mut counter = 0;

    for e in stdin().events() {
        match e.unwrap() {
            Event::Key(Key::Char('q')) => break,
            Event::Key(Key::Up) => {
                write!(screens.write(), "{}", counter).unwrap();
                screens.switch();
                counter += 1;
            }
            _ => {}
        }
    }
}
