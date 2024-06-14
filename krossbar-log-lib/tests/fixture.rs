use std::{path::PathBuf, time::Duration};

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
    logs_dir: TempDir,
    cancel_token: CancellationToken,
}

impl Fixture {
    pub async fn new() -> Self {
        let socket_dir =
            TempDir::new("logger_socket_dir").expect("Failed to create socket tempdir");

        let socket_path: PathBuf = socket_dir.path().join("krossbar_logger.socket");

        let logs_dir = TempDir::new("logs").expect("Failed to create tempdir");

        let this = Self {
            socket_path,
            _socket_dir: socket_dir,
            logs_dir,
            cancel_token: CancellationToken::new(),
        };

        this.start_logger().await;
        this
    }

    async fn start_logger(&self) {
        let args = Args {
            keep_num_files: 10,
            num_bytes_rotate: 1_000_000,
            log_location: self
                .logs_dir
                .path()
                .join("krossbar.log")
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
}

#[fixture]
pub async fn make_fixture() -> Fixture {
    let fixture = Fixture::new().await;

    // Wait for logger to start
    tokio::time::sleep(Duration::from_millis(1)).await;
    init_client_logger(fixture.logger_socket_path().clone()).await;

    fixture
}

async fn init_client_logger(logger_sock: PathBuf) {
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

    log::set_boxed_logger(logger)
        .map(|()| log::set_max_level(LevelFilter::Debug))
        .unwrap();
}
