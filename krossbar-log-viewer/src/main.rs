pub mod colorizer;
pub mod log_directory_entry;
pub mod log_directory_reader;
pub mod log_files;
pub mod screen;

use std::{io::stdin, path::PathBuf, sync::mpsc, thread};

use clap::{self, Parser};
use krossbar_log_viewer::{
    colorizer::Colorizer, log_files::log_file_trait::ShiftDirection, log_registry::LogRegistry,
};
use log::LevelFilter;

use krossbar_log_common::DEFAULT_LOG_LOCATION;
use notify::{Error, EventKind, RecursiveMode, Watcher};
use screen::Screen;
use termion::{event::Key, input::TermRead};

/// Krossbar log viewer
#[derive(Parser, Debug, Clone)]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// Log level: OFF, ERROR, WARN, INFO, DEBUG, TRACE
    #[clap(short, long, value_parser, default_value_t = LevelFilter::Info)]
    pub log_level: log::LevelFilter,

    /// Log files location
    #[clap(long, value_parser, default_value_t = DEFAULT_LOG_LOCATION.into())]
    pub log_location: String,

    /// Output appended data as the file grows
    #[clap(short, long, value_parser, default_value_t = false)]
    pub follow: bool,
}

fn render(
    screen: &mut Screen,
    registry: &mut LogRegistry,
    colorizer: &mut Colorizer,
    direction: ShiftDirection,
    shift: usize,
) {
    let (_, h) = Screen::size();
    registry.shift(direction, shift, h as usize - 2);
    registry.write_io(&mut screen.write(), colorizer);
}

fn follow(args: Args, mut screen: Screen, mut registry: LogRegistry, mut colorizer: Colorizer) {
    let (sender, receiver) = mpsc::channel();

    let mut watcher = notify::recommended_watcher(sender.clone()).unwrap();

    thread::spawn(move || {
        let stdin = stdin();

        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') | Key::Ctrl('c') => {
                    sender.send(Err(Error::generic("Shutdown"))).unwrap();
                    return;
                }
                _ => {}
            }
        }
    });

    watcher
        .watch(
            &PathBuf::from(&args.log_location),
            RecursiveMode::NonRecursive,
        )
        .unwrap();

    for event in receiver {
        match event {
            Ok(event) if matches!(event.kind, EventKind::Modify(_)) => {
                let (_, h) = Screen::size();

                render(
                    &mut screen,
                    &mut registry,
                    &mut colorizer,
                    ShiftDirection::Right,
                    h as usize,
                );
            }
            Ok(_) => {}
            Err(_) => break,
        }
    }
}

fn interactive(mut screen: Screen, mut registry: LogRegistry, mut colorizer: Colorizer) {
    let stdin = stdin();

    for c in stdin.keys() {
        match c.unwrap() {
            Key::Char('q') | Key::Ctrl('c') => break,
            Key::Up => {
                render(
                    &mut screen,
                    &mut registry,
                    &mut colorizer,
                    ShiftDirection::Left,
                    1,
                );
            }
            Key::Down => {
                render(
                    &mut screen,
                    &mut registry,
                    &mut colorizer,
                    ShiftDirection::Right,
                    1,
                );
            }
            Key::PageUp => {
                let (_, h) = Screen::size();
                render(
                    &mut screen,
                    &mut registry,
                    &mut colorizer,
                    ShiftDirection::Left,
                    h as usize,
                );
            }
            Key::PageDown => {
                let (_, h) = Screen::size();
                render(
                    &mut screen,
                    &mut registry,
                    &mut colorizer,
                    ShiftDirection::Right,
                    h as usize,
                );
            }
            _ => {}
        }
    }
}

fn main() {
    let args = Args::parse();

    let mut screen = Screen::new();
    let mut registry = LogRegistry::new(&args.log_location);
    let mut colorizer = Colorizer::new();

    render(
        &mut screen,
        &mut registry,
        &mut colorizer,
        ShiftDirection::Left,
        0,
    );

    if args.follow {
        follow(args, screen, registry, colorizer)
    } else {
        interactive(screen, registry, colorizer);
    }
}
