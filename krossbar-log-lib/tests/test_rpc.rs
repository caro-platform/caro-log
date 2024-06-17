use std::time::Duration;

use log::*;
use rstest::rstest;

mod fixture;
use fixture::{make_fixture, Fixture};

#[rstest]
#[awt]
#[tokio::test(flavor = "multi_thread")]
async fn test_simple_log(
    #[from(make_fixture)]
    #[future]
    fixture: Fixture,
) {
    error!("Error message");
    warn!("Warning message");
    info!("Info message");
    debug!("Debug message");

    // Wait for logger to write file
    tokio::time::sleep(Duration::from_millis(1)).await;

    let log_file_text = std::fs::read_to_string(fixture.log_file_path()).unwrap();
    eprintln!("Log text:\n{log_file_text}");

    assert!(log_file_text.contains("Error message"));
    assert!(log_file_text.contains("Warning message"));
    assert!(log_file_text.contains("Info message"));
    assert!(log_file_text.contains("Debug message"));

    fixture.cancel();
}
