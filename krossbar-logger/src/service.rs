use std::{collections::HashMap, path::Path, sync::Arc};

use futures::{channel::mpsc::Receiver, lock::Mutex, select, FutureExt, StreamExt};
use log::{debug, error, warn};

use krossbar_bus_common::DEFAULT_HUB_SOCKET_PATH;
use krossbar_bus_lib::{Service, Signal};
use krossbar_log_common::logger_interface::{
    SetLogLevel, LOGGER_SERVICE_NAME, LOG_CLIENTS_METHOD_NAME, ROTATED_SIGNAL,
    SET_LOG_LEVEL_METHOD_NAME,
};

use krossbar_rpc::writer::RpcWriter;

use crate::logger::Event;

type ClientRegistryType = Arc<Mutex<HashMap<String, RpcWriter>>>;

struct ServiceEndpoints {
    service: Service,
    rotate_signal: Signal<String>,
}

pub struct LoggerService;

impl LoggerService {
    pub async fn run(clients: ClientRegistryType, mut event_receiver: Receiver<Event>) {
        tokio::spawn(async move {
            let ServiceEndpoints {
                mut service,
                rotate_signal,
            } = Self::connect(clients).await;

            select! {
                _ = service.poll().fuse() => {},
            event = event_receiver.next() => {
                if event.is_none() {
                    error!("Event channel sender is closed");
                }

                match event.unwrap() {
                    Event::Rotated(file_name) => {
                        if let Err(e) = rotate_signal.emit(file_name).await {
                            warn!("Failed to send 'rotated' event: {e:?}");
                        }
                    }
                }
            }}
        });
    }

    async fn connect(clients: ClientRegistryType) -> ServiceEndpoints {
        debug!("Connecting logger service");

        let mut service = Service::new(LOGGER_SERVICE_NAME, Path::new(DEFAULT_HUB_SOCKET_PATH))
            .await
            .unwrap();

        let rotate_signal = service.register_signal(ROTATED_SIGNAL).unwrap();

        Self::register_set_log_level(&mut service, clients.clone());
        Self::register_get_clients(&mut service, clients.clone());

        ServiceEndpoints {
            rotate_signal,
            service,
        }
    }

    fn register_set_log_level(service: &mut Service, clients: ClientRegistryType) {
        service
            .register_async_method(
                SET_LOG_LEVEL_METHOD_NAME,
                move |_service, message: SetLogLevel| {
                    let clients = clients.clone();

                    async move {
                        if let Some(writer) = clients.lock().await.get_mut(&message.service_name) {
                            let _ = writer
                                .send_message(SET_LOG_LEVEL_METHOD_NAME, &message.level)
                                .await;
                        }
                    }
                },
            )
            .unwrap();
    }

    fn register_get_clients(service: &mut Service, clients: ClientRegistryType) {
        service
            .register_async_method(LOG_CLIENTS_METHOD_NAME, move |_service, _message: ()| {
                let clients = clients.clone();

                async move {
                    let clients: Vec<String> = clients.lock().await.keys().cloned().collect();

                    clients
                }
            })
            .unwrap();
    }
}
