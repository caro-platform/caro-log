use std::{collections::VecDeque, fs::OpenOptions, io::Write, path::Path};

use chrono::NaiveDateTime;
use karo_log_viewer::{
    log_file_trait::{LogFile, ShiftDirection},
    rotated_log_file::RotatedLogFile,
};
use log::LevelFilter;
use tempdir::TempDir;

fn write_log(log_file_path: &Path) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(log_file_path)
        .unwrap();

    file.write_all("log0\nlog1\nlog2\nlog3\nlog4".as_bytes())
        .unwrap();
}

#[test]
fn test_rotated_log() {
    let _ = pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Debug)
        .try_init();

    let log_dir = TempDir::new("karo_log_dir").expect("Failed to create log tempdir");
    let log_file_path = log_dir.path().join("test.log");

    write_log(&log_file_path);

    let mut rotated = RotatedLogFile::new(
        log_file_path.to_path_buf(),
        NaiveDateTime::from_timestamp(0, 42_000_000),
    );

    assert_eq!(rotated.lines(), &VecDeque::from_iter([]));

    // [0, 1, x, x, x]
    rotated.shift_and_read(ShiftDirection::Right, 2, 0);
    assert_eq!(
        rotated.lines(),
        &VecDeque::from_iter(["log0".to_owned(), "log1".to_owned()])
    );

    // [x, x, 2, 3, x]
    rotated.shift_and_read(ShiftDirection::Right, 2, 2);
    assert_eq!(
        rotated.lines(),
        &VecDeque::from_iter(["log2".to_owned(), "log3".to_owned()])
    );

    // [x, 1, 2, x, x]
    rotated.shift_and_read(ShiftDirection::Left, 2, 1);
    assert_eq!(
        rotated.lines(),
        &VecDeque::from_iter(["log1".to_owned(), "log2".to_owned()])
    );

    // [x, 1, 2, 3, x]
    rotated.shift_and_read(ShiftDirection::Right, 3, 0);
    assert_eq!(
        rotated.lines(),
        &VecDeque::from_iter(["log1".to_owned(), "log2".to_owned(), "log3".to_owned()])
    );

    // [x, 1, 2, x, x]
    rotated.shift_and_read(ShiftDirection::Right, 2, 0);
    assert_eq!(
        rotated.lines(),
        &VecDeque::from_iter(["log1".to_owned(), "log2".to_owned(), "log3".to_owned()])
    );

    // [x, 1, 2, 3, x]
    rotated.shift_and_read(ShiftDirection::Right, 3, 0);
    assert_eq!(
        rotated.lines(),
        &VecDeque::from_iter(["log1".to_owned(), "log2".to_owned(), "log3".to_owned()])
    );

    // [x, x, x, 3, 4]
    rotated.shift_and_read(ShiftDirection::Right, 3, 2);
    assert_eq!(
        rotated.lines(),
        &VecDeque::from_iter(["log3".to_owned(), "log4".to_owned()])
    );

    // [x, x, 2, 3, 4]
    rotated.shift_and_read(ShiftDirection::Left, 3, 0);
    assert_eq!(
        rotated.lines(),
        &VecDeque::from_iter(["log2".to_owned(), "log3".to_owned(), "log4".to_owned()])
    );

    // [x, x, x, x, x]
    rotated.rev();
    assert_eq!(rotated.lines(), &VecDeque::from_iter([]));

    // [x, 1, 2, 3, x]
    rotated.shift_and_read(ShiftDirection::Left, 3, 1);
    assert_eq!(
        rotated.lines(),
        &VecDeque::from_iter(["log1".to_owned(), "log2".to_owned(), "log3".to_owned()])
    );
}
