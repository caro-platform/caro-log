use std::{fs::File, path::Path};

use chrono::DateTime;
use log::LevelFilter;
use tempdir::TempDir;

use karo_log_common::ROTATED_LOG_TIMESTAMP_FORMAT;
use karo_log_viewer::log_directory_reader::DirectoryReader;

fn create_log_files(path: &Path) {
    File::create(path.join("invalid.log")).unwrap();

    File::create(path.join(&format!(
            "karo_{}.log",
            DateTime::parse_from_rfc2822("Wed, 18 Feb 2015 23:16:09 GMT")
                .unwrap()
                .format(ROTATED_LOG_TIMESTAMP_FORMAT))))
    .unwrap();

    File::create(path.join(&format!(
        "karo_{}.log",
        DateTime::parse_from_rfc2822("Wed, 18 Feb 2015 11:16:09 GMT")
            .unwrap()
            .format(ROTATED_LOG_TIMESTAMP_FORMAT))))
    .unwrap();

    File::create(path.join("invalid2.log")).unwrap();

    File::create(path.join(&format!(
        "karo_{}.log",
        DateTime::parse_from_rfc2822("Tue, 17 Feb 2015 23:16:09 GMT")
            .unwrap()
            .format(ROTATED_LOG_TIMESTAMP_FORMAT))))
    .unwrap();

    File::create(path.join(&format!(
    "karo_{}.log",
    DateTime::parse_from_rfc2822("Mon, 23 Feb 2015 01:00:00 GMT")
        .unwrap()
        .format(ROTATED_LOG_TIMESTAMP_FORMAT))))
    .unwrap();

    File::create(path.join("karo.log")).unwrap();
}

#[test]
fn dir_reader_test() {
    let _ = pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Debug)
        .try_init();

    let log_dir = TempDir::new("karo_log_dir").expect("Failed to create log tempdir");

    create_log_files(log_dir.path());

    let live_log_location: String = log_dir.path().join("karo.log").to_string_lossy().into();
    let mut read_result = DirectoryReader::read_dir_logs(&live_log_location).into_iter();

    let mut first_file = read_result.next().unwrap();
    assert_eq!(first_file.log_file_name, "karo_2015_02_17_23_16_09.log");

    first_file = read_result.next().unwrap();
    assert_eq!(first_file.log_file_name, "karo_2015_02_18_11_16_09.log");

    first_file = read_result.next().unwrap();
    assert_eq!(first_file.log_file_name, "karo_2015_02_18_23_16_09.log");

    first_file = read_result.next().unwrap();
    assert_eq!(first_file.log_file_name, "karo_2015_02_23_01_00_00.log");

    first_file = read_result.next().unwrap();
    assert_eq!(first_file.log_file_name, "karo.log");

    assert!(read_result.next().is_none());
}
