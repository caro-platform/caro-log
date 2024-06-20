use std::{collections::HashMap, path::Path, sync::Arc};

use futures::lock::Mutex;

use krossbar_bus_common::DEFAULT_HUB_SOCKET_PATH;
use krossbar_bus_lib::{Service, Signal};
use krossbar_log_common::logger_interface::{
    SetLogLevel, LOGGER_SERVICE_NAME, LOG_CLIENTS_METHOD_NAME, ROTATED_SIGNAL,
    SET_LOG_LEVEL_METHOD_NAME,
};
use krossbar_rpc::writer::RpcWriter;

type ClientRegistryType = Arc<Mutex<HashMap<String, RpcWriter>>>;

struct ServiceEndpoints {
    service: Service,
    rotate_signal: Signal<String>,
}

pub struct LoggerService {
    service: Option<ServiceEndpoints>,
    clients: ClientRegistryType,
}

impl LoggerService {
    pub fn new(clients: ClientRegistryType) -> Self {
        Self {
            service: None,
            clients,
        }
    }

    pub async fn run(&mut self) {
        if self.service.is_none() {
            self.service = Some(self.connect().await);
        }

        let _ = self.service.as_mut().unwrap().service.poll().await;
    }

    pub async fn on_rotated(&mut self, path: &str) {
        if let Some(ServiceEndpoints {
            ref mut rotate_signal,
            ..
        }) = self.service
        {
            let _ = rotate_signal.emit(path.into()).await;
        }
    }

    async fn connect(&mut self) -> ServiceEndpoints {
        let mut service = Service::new(LOGGER_SERVICE_NAME, Path::new(DEFAULT_HUB_SOCKET_PATH))
            .await
            .unwrap();

        let rotate_signal = service.register_signal(ROTATED_SIGNAL).unwrap();

        Self::register_set_log_level(&mut service, self.clients.clone());
        Self::register_get_clients(&mut service, self.clients.clone());

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
