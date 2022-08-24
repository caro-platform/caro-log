pub mod log_directory_entry;
pub mod log_directory_reader;
pub mod log_files;
pub mod screen;

use std::io::stdin;

use clap::{self, Parser};
use karo_log_viewer::{log_files::log_file_trait::ShiftDirection, log_registry::LogRegistry};
use log::LevelFilter;

use karo_log_common::DEFAULT_LOG_LOCATION;
use screen::Screen;
use termion::{event::Key, input::TermRead};

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

fn render(
    screen: &mut Screen,
    registry: &mut LogRegistry,
    direction: ShiftDirection,
    shift: usize,
) {
    let (_, h) = Screen::size();
    registry.shift(direction, shift, h as usize);
    registry.write_io(&mut screen.write());
}

fn main() {
    let args = Args::parse();

    let mut screen = Screen::new();
    let mut registry = LogRegistry::new(&args.log_location);

    render(&mut screen, &mut registry, ShiftDirection::Left, 0);

    for c in stdin().keys() {
        match c.unwrap() {
            Key::Char('q') => break,
            Key::Up => {
                render(&mut screen, &mut registry, ShiftDirection::Left, 1);
            }
            Key::Down => {
                render(&mut screen, &mut registry, ShiftDirection::Right, 1);
            }
            Key::PageUp => {
                let (_, h) = Screen::size();
                render(&mut screen, &mut registry, ShiftDirection::Left, h as usize);
            }
            Key::PageDown => {
                let (_, h) = Screen::size();
                render(
                    &mut screen,
                    &mut registry,
                    ShiftDirection::Right,
                    h as usize,
                );
            }
            _ => {}
        }
    }
}
