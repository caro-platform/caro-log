use std::{fs::OpenOptions, io::Write, path::Path};

use log::LevelFilter;
use tempdir::TempDir;

use karo_log_viewer::{log_file_trait::ShiftDirection, log_registry::LogRegistry};

fn write_log(log_file_path: &Path, num: usize) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(log_file_path)
        .unwrap();

    file.write_all(format!("log{n}0\nlog{n}1\nlog{n}2\nlog{n}3\nlog{n}4", n = num).as_bytes())
        .unwrap();
}

#[test]
fn test_single_file() {
    const WINDOW_SIZE: usize = 3;

    let _ = pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Debug)
        .try_init();

    let log_dir = TempDir::new("karo_log_dir").expect("Failed to create log tempdir");
    let log_file_path = log_dir.path().join("karo.log");

    write_log(&log_file_path, 0);
    let mut buffer = String::new();

    let mut registry = LogRegistry::new(&log_file_path.to_string_lossy());
    registry.shift(ShiftDirection::Left, 0, WINDOW_SIZE);
    registry.write(&mut buffer);
    assert_eq!(buffer, "log02\nlog03\nlog04");
}
