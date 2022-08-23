use std::{fs::OpenOptions, io::Write, path::Path};

use log::LevelFilter;
use tempdir::TempDir;

use karo_log_viewer::{log_files::log_file_trait::ShiftDirection, log_registry::LogRegistry};

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
fn test_single_file_registry() {
    const WINDOW_SIZE: usize = 3;

    let _ = pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Debug)
        .try_init();

    let log_dir = TempDir::new("karo_log_dir").expect("Failed to create log tempdir");
    let log_file_path = log_dir.path().join("karo.log");

    write_log(&log_file_path, 0);
    let mut buffer = String::new();

    // [x, x, 2, 3, 4]
    let mut registry = LogRegistry::new(&log_file_path.to_string_lossy());
    registry.shift(ShiftDirection::Left, 0, WINDOW_SIZE);
    registry.write(&mut buffer);
    assert_eq!(buffer, "log02\nlog03\nlog04");

    // [x, 1, 2, 3, x]
    registry.shift(ShiftDirection::Left, 1, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log01\nlog02\nlog03");

    // [x, 1, 2, x, x]
    registry.shift(ShiftDirection::Left, 1, WINDOW_SIZE - 1);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log01\nlog02");

    // [x, 1, 2, 3, x]
    registry.shift(ShiftDirection::Right, 0, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log01\nlog02\nlog03");

    // [0, 1, 2, x, x]
    registry.shift(ShiftDirection::Left, 1, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log00\nlog01\nlog02");

    // [x, 1, 2, 3, x]
    registry.shift(ShiftDirection::Right, 1, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log01\nlog02\nlog03");

    // [0, 1, 2, x, x]
    registry.shift(ShiftDirection::Left, 2, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log00\nlog01\nlog02");

    // [x, 1, 2, 3, x]
    registry.shift(ShiftDirection::Right, 1, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log01\nlog02\nlog03");
}

#[test]
fn test_multiple_files_registry() {
    const WINDOW_SIZE: usize = 5;

    let _ = pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Debug)
        .try_init();

    let log_dir = TempDir::new("karo_log_dir").expect("Failed to create log tempdir");

    let live_log_file_path = log_dir.path().join("karo.log");
    write_log(&live_log_file_path, 0);

    let log_file_path = log_dir.path().join("karo_2022_08_22_18_00_00.log");
    write_log(&log_file_path, 1);

    let log_file_path = log_dir.path().join("karo_2022_08_22_11_00_00.log");
    write_log(&log_file_path, 2);

    let mut buffer = String::new();

    // [x, x, 2, 3, 4]
    let mut registry = LogRegistry::new(&live_log_file_path.to_string_lossy());
    registry.shift(ShiftDirection::Left, 0, 3);
    registry.write(&mut buffer);
    assert_eq!(buffer, "log02\nlog03\nlog04");

    // [..][0, 1, 2, 3, 4]
    registry.shift(ShiftDirection::Left, 0, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log00\nlog01\nlog02\nlog03\nlog04");

    // [x, x, x, 3, 4][0, 1, 2, x, x]
    registry.shift(ShiftDirection::Left, 2, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log13\nlog14\nlog00\nlog01\nlog02");

    // [x, x, x, x, 4][0, 1, 2, 3, x]
    registry.shift(ShiftDirection::Right, 1, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log14\nlog00\nlog01\nlog02\nlog03");

    // [x, 1, 2, 3, 4][0, x, x, x, x]
    registry.shift(ShiftDirection::Left, 3, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log11\nlog12\nlog13\nlog14\nlog00");

    // [0, 1, 2, 3, 4][..]
    registry.shift(ShiftDirection::Left, 1, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log10\nlog11\nlog12\nlog13\nlog14");

    // [x, x, 2, 3, 4][0, 1, x, x, x][..]
    registry.shift(ShiftDirection::Left, 3, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log22\nlog23\nlog24\nlog10\nlog11");

    // [0, 1, 2, 3, 4][..][..]
    registry.shift(ShiftDirection::Left, 5, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log20\nlog21\nlog22\nlog23\nlog24");

    // [..][..][x, x, 2, 3, 4]
    registry.shift(ShiftDirection::Right, 12, WINDOW_SIZE);
    buffer.clear();
    registry.write(&mut buffer);
    assert_eq!(buffer, "log00\nlog01\nlog02\nlog03\nlog04");
}
