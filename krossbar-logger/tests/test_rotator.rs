use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
    time::Duration,
};

use log::LevelFilter;
use tempdir::TempDir;

use krossbar_logger_lib::{args::Args, rotator::Rotator};

fn make_args(log_dir: &TempDir) -> Args {
    let log_location: String = log_dir
        .path()
        .join("krossbar_log.messages")
        .as_os_str()
        .to_str()
        .unwrap()
        .into();

    Args {
        log_level: LevelFilter::Debug,
        log_location,
        num_bytes_rotate: u64::MAX,
        // Keep single rotated file
        keep_num_files: 1,
    }
}

fn write_log_message(message: &str, log_path: &str) {
    let mut log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .open(log_path)
        .unwrap();

    log_file.write_all(message.as_bytes()).unwrap();
}

fn read_log_content(log_path: &str) -> String {
    let mut log_file = OpenOptions::new().read(true).open(log_path).unwrap();

    let mut buf: Vec<u8> = vec![];
    log_file.read_to_end(&mut buf).unwrap();

    String::from_utf8(buf).unwrap()
}

fn log_dir_files(path: PathBuf) -> Vec<String> {
    path.parent()
        .unwrap()
        .read_dir()
        .unwrap()
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(dir_entry) => {
                let metadata = dir_entry.metadata();
                if metadata.is_ok() && metadata.unwrap().is_file() {
                    Some(dir_entry.path().to_string_lossy().into_owned())
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect()
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rotator() {
    let log_dir = TempDir::new("krossbar_log_dir").expect("Failed to create log tempdir");

    let args = make_args(&log_dir);

    write_log_message("Log0", &args.log_location);
    assert_eq!(&read_log_content(&args.log_location), "Log0");

    // First rotate
    Rotator::new(args.keep_num_files, PathBuf::from(&args.log_location)).rotate();

    write_log_message("Log1", &args.log_location);

    let mut log_files = log_dir_files(PathBuf::from(&args.log_location));

    assert_eq!(log_files.len(), 2);
    log_files.sort();

    println!("Rotated file path {}", log_files[1]);
    assert_eq!(&read_log_content(&log_files[1]), "Log0");
    assert_eq!(&read_log_content(&args.log_location), "Log1");

    // Sleep to make different names
    std::thread::sleep(Duration::from_secs(1));
    // Second rotate
    Rotator::new(args.keep_num_files, PathBuf::from(&args.log_location)).rotate();

    write_log_message("Log2", &args.log_location);

    let mut log_files = log_dir_files(PathBuf::from(&args.log_location));

    assert_eq!(log_files.len(), 2);
    log_files.sort();

    println!("Rotated file path {}", log_files[1]);
    assert_eq!(&read_log_content(&log_files[1]), "Log1");
    assert_eq!(&read_log_content(&args.log_location), "Log2");
}
