use std::path::PathBuf;

use log::LevelFilter;
use rstest::fixture;
use tempdir::TempDir;
use tokio_util::sync::CancellationToken;

use krossbar_log_lib::Logger as ClientLogger;
use krossbar_logger_lib::{args::Args, logger::Logger};

pub struct Fixture {
    socket_path: PathBuf,
    // Need this to keep temp dir from deletion
    _socket_dir: TempDir,
    _logs_dir: TempDir,
    log_file_path: PathBuf,
    cancel_token: CancellationToken,
}

impl Fixture {
    pub fn new() -> Self {
        let socket_dir =
            TempDir::new("logger_socket_dir").expect("Failed to create socket tempdir");

        let socket_path: PathBuf = socket_dir.path().join("krossbar_logger.socket");

        let logs_dir = TempDir::new("logs").expect("Failed to create tempdir");
        let log_file_path = logs_dir.path().join("krossbar.log");

        Self {
            socket_path,
            _socket_dir: socket_dir,
            _logs_dir: logs_dir,
            log_file_path,
            cancel_token: CancellationToken::new(),
        }
    }

    pub async fn start_logger(&self) {
        let args = Args {
            keep_num_files: 10,
            num_bytes_rotate: 1_000_000,
            log_location: self
                .log_file_path
                .to_owned()
                .into_os_string()
                .into_string()
                .unwrap(),
            log_level: LevelFilter::Trace,
        };

        let token = self.cancel_token.clone();
        let socket_path: PathBuf = self.socket_path.clone().into();
        tokio::spawn(async move {
            tokio::select! {
                _ = Logger::new(args, socket_path).run() => {}
                _ = token.cancelled() => {}
            }
        });
    }

    pub fn cancel(&self) {
        self.cancel_token.cancel()
    }

    pub fn logger_socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    pub fn log_file_path(&self) -> &PathBuf {
        &self.log_file_path
    }
}

#[fixture]
pub fn make_fixture() -> Fixture {
    Fixture::new()
}

pub async fn init_client_logger(logger_sock: PathBuf) {
    let logger = Box::new(
        ClientLogger::new(
            "test.log.service",
            LevelFilter::Debug,
            true,
            Some(logger_sock),
        )
        .await
        .unwrap(),
    );

    tokio::spawn(logger.run());
}
