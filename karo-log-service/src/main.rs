use std::sync::Arc;

use clap::{self, Parser};
use tokio::{
    fs::{File, OpenOptions},
    io::AsyncWriteExt,
    sync::Mutex,
};

use karo_bus_lib::Bus;

use karo_log_common::{
    log_message::LogMessage, DEFAULT_LOG_LOCATION, LOGGING_METHOD_NAME, LOGGING_SERVICE_NAME,
};
/// Karo bus monitor
#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub struct Args {
    /// Service to monitor
    #[clap(short, long, value_parser, default_value_t = DEFAULT_LOG_LOCATION.into())]
    pub log_location: String,
}

async fn handle_message(log_file: Arc<Mutex<File>>, message: LogMessage) {
    let bytes = message.message.as_bytes();
    if let Err(err) = log_file.lock().await.write_all(bytes).await {
        eprintln!("Failed to write log message: {}", err.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Karo logging service");

    let args = Args::parse();

    let log_file = Arc::new(Mutex::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&args.log_location)
            .await?,
    ));

    let mut bus = Bus::register(LOGGING_SERVICE_NAME)
        .await
        .expect("Failed to register logging service");

    bus.register_method(LOGGING_METHOD_NAME, move |message: LogMessage| {
        let log_file = log_file.clone();
        async {
            handle_message(log_file, message).await;
        }
    })
    .expect("Failed to register logging function");

    println!("Succesfully started logging service. Listening for message");
    let _ = tokio::signal::ctrl_c().await;
    println!("Shutting down");

    Ok(())
}
