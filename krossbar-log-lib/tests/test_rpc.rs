use std::time::Duration;

use fork::{fork, Fork};
use log::*;
use rstest::rstest;

mod fixture;
use fixture::{init_client_logger, make_fixture, Fixture};
use tokio::runtime::Runtime;

#[rstest]
fn test_simple_log(#[from(make_fixture)] fixture: Fixture) {
    // Logger has it's own log, so we need to fork here to set client logger
    match fork() {
        Ok(Fork::Child) => {
            let rt = Runtime::new().unwrap();

            rt.block_on(async move {
                println!("Starting logger");
                fixture.start_logger().await;
                tokio::time::sleep(Duration::from_millis(99)).await;
                println!("Logger exited");
                fixture.cancel();
            })
        }
        Ok(Fork::Parent(child)) => {
            eprintln!("Child PID: {child}");

            let rt = Runtime::new().unwrap();

            rt.block_on(async move {
                // Wait for logger to start
                tokio::time::sleep(Duration::from_millis(10)).await;

                println!("Initializing client logger");
                init_client_logger(fixture.logger_socket_path().clone()).await;

                error!("Error message");
                warn!("Warning message");
                info!("Info message");
                debug!("Debug message");

                // Wait for logger to write file
                tokio::time::sleep(Duration::from_millis(1)).await;

                let log_file_text = std::fs::read_to_string(fixture.log_file_path()).unwrap();
                println!("Log text:\n{log_file_text}");

                assert!(log_file_text.contains("Error message"));
                assert!(log_file_text.contains("Warning message"));
                assert!(log_file_text.contains("Info message"));
                assert!(log_file_text.contains("Debug message"));

                fixture.cancel();
            });
        }
        Err(e) => panic!("Failed to fork: {e}"),
    }
}
