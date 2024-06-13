use std::time::Duration;

use log::{debug, info, warn, LevelFilter};
use tempdir::TempDir;

use krossbar_logger_lib::{args::Args, logger::Logger};

fn make_args(log_dir: &TempDir) -> Args {
    let log_location: String = log_dir
        .path()
        .join("krossbar_log.messages")
        .into_os_string()
        .into_string()
        .unwrap();

    println!("Log location: {log_location}");

    Args {
        log_level: LevelFilter::Debug,
        log_location,
        num_bytes_rotate: u64::MAX,
        // Keep single rotated file
        keep_num_files: 1,
    }
}

#[tokio::test]
async fn test_self_logger() {
    let log_dir = TempDir::new("krossbar_log_dir").expect("Failed to create log tempdir");
    let socket_path = log_dir.path().join("logger.sock");

    let args = make_args(&log_dir);

    let logger = Logger::new(args, socket_path);
    tokio::spawn(logger.run());

    debug!("Test {}", "DEBUG");
    info!("Test {}", "INFO");
    warn!("Test {}", "WARN");

    tokio::time::sleep(Duration::from_millis(1)).await;

    let log_content =
        std::fs::read_to_string(log_dir.path().join("krossbar_log.messages")).unwrap();

    assert!(log_content.contains("Test DEBUG"));
    assert!(log_content.contains("Test INFO"));
    assert!(log_content.contains("Test WARN"));
}
