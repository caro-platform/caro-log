use karo_bus_lib::Bus;

use karo_log_common::{CONTROL_SERVICE_NAME, SET_LOG_LEVEL_METHOD_NAME};
use log::LevelFilter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bus = Bus::register(CONTROL_SERVICE_NAME)
        .await
        .expect("Failed to register logging service");

    let client = bus.connect("peer_service_name").await.unwrap();
    client
        .call::<LevelFilter, ()>(SET_LOG_LEVEL_METHOD_NAME, &LevelFilter::Debug)
        .await
        .unwrap();

    Ok(())
}
