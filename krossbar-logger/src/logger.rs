use std::{
    collections::HashMap, fs, os::unix::fs::PermissionsExt, path::PathBuf, pin::Pin, sync::Arc,
};

use futures::{
    channel::mpsc::{channel, Receiver, Sender},
    future::{pending, FutureExt as _},
    lock::Mutex,
    stream::FuturesUnordered,
    Future, SinkExt, StreamExt as _,
};

use krossbar_log_common::logger_interface::REGISTER_METHOD_NAME;
use log::{debug, error, info, warn};
use tokio::{
    net::{
        unix::{self, UCred},
        UnixListener,
    },
    select,
};

use krossbar_rpc::{request::RpcRequest, rpc::Rpc, writer::RpcWriter, Error, Result};
use krossbar_state_machine::Machine;

use crate::{args::Args, client::Client, service::LoggerService, writer::Writer, LogEvent};

use crate::self_logger::SelfLogger;
use log::set_boxed_logger;

const CHANNEL_SIZE: usize = 100;

type TasksMapType = FuturesUnordered<Pin<Box<dyn Future<Output = Option<String>> + Send>>>;
type ClientRegistryType = Arc<Mutex<HashMap<String, RpcWriter>>>;

pub enum Event {
    Rotated(String),
}

pub struct Logger {
    tasks: TasksMapType,
    socket_path: PathBuf,
    clients: ClientRegistryType,
    log_receiver: Receiver<LogEvent>,
    log_sender: Sender<LogEvent>,
    writer: Writer,
}

impl Logger {
    pub fn new(args: Args, socket_path: PathBuf) -> Self {
        let tasks: TasksMapType = FuturesUnordered::new();
        tasks.push(Box::pin(pending()));

        let (log_sender, log_receiver) = channel(CHANNEL_SIZE);

        set_boxed_logger(Box::new(SelfLogger::new(
            args.log_level,
            log_sender.clone(),
        )))
        .map(|()| log::set_max_level(args.log_level))
        .unwrap();

        let clients = Arc::new(Mutex::new(HashMap::new()));

        Self {
            tasks,
            socket_path,
            clients: clients.clone(),
            log_receiver,
            log_sender,
            writer: Writer::new(&args),
        }
    }

    /// Hub main loop
    pub async fn run(mut self) {
        println!("Logger socket path: {:?}", self.socket_path);

        // Remove hanging socket if present
        let _ = std::fs::remove_file(&self.socket_path);

        let listener = UnixListener::bind(&self.socket_path).unwrap();

        println!("Logger started listening for new connections");

        // Update permissions to be accessible for th eclient
        let socket_permissions = fs::Permissions::from_mode(0o666);
        fs::set_permissions(&self.socket_path, socket_permissions).unwrap();

        let (mut event_sender, event_receiver) = channel(CHANNEL_SIZE);

        LoggerService::run(self.clients.clone(), event_receiver).await;

        async move {
            loop {
                select! {
                    // Accept new connection requests
                    client = listener.accept().fuse() => {
                        match client {
                            Ok((stream, _)) => {
                                let credentials = stream.peer_cred();
                                let rpc = Rpc::new(stream, "");

                                match credentials {
                                    Ok(credentials) => {
                                        info!("New connection request: {credentials:?}");

                                        let client_machine = Machine::init((rpc, credentials, self.clients.clone(), self.log_sender.clone()))
                                            .then(Self::authorize)
                                            .then(Client::run)
                                            .unwrap(Self::client_name);

                                        self.tasks.push(Box::pin(client_machine))
                                    },
                                    Err(e) => {
                                        warn!("Failed to get client creadentials: {}", e.to_string());
                                    }
                                }

                            },
                            Err(e) => {
                                warn!("Failed client connection attempt: {}", e.to_string())
                            }
                        }
                    },
                    // Loop clients. Loop return means a client is disconnected
                    disconnected_service = self.tasks.next() => {
                        let service_name = disconnected_service.unwrap();

                        match service_name {
                            Some(service_name) => {
                                debug!("Client disconnected: {}", service_name);

                                self.clients.lock().await.remove(&service_name);
                            }
                            _ => {
                                debug!("Anonymous client disconnected");
                            }
                        }
                    },
                    log_message = self.log_receiver.next() => {
                        match log_message {
                            Some(message) => {if let Some(rotated_file) = self.writer.log_message(message) {
                                if event_sender.send(Event::Rotated(rotated_file)).await.is_err() {
                                    error!("Event channel receiver is closed");
                                }
                            }},
                            _ => warn!("Failed to receive log message through the channel")
                        }
                    },
                    _ = tokio::signal::ctrl_c().fuse() => return
                }
            }
        }
        .await;

        // Cleanup socket
        let _ = std::fs::remove_file(&self.socket_path);
    }

    async fn authorize(
        (mut rpc, credentials, clients, log_sender): (
            Rpc,
            UCred,
            ClientRegistryType,
            Sender<LogEvent>,
        ),
    ) -> std::result::Result<(unix::pid_t, String, Rpc, Sender<LogEvent>), ()> {
        debug!("New client connection. Waiting for an auth message");

        // Authorize the client
        let service_name = match rpc.poll().await {
            Some(mut request) => {
                if request.endpoint() != REGISTER_METHOD_NAME {
                    request
                        .respond::<()>(Err(Error::InternalError(format!(
                            "Expected registration call from a client. Got {}",
                            request.endpoint()
                        ))))
                        .await;
                }

                match request.take_body().unwrap() {
                    // Valid call message
                    krossbar_rpc::request::Body::Call(bson) => {
                        // Valid Auth message
                        match bson::from_bson::<String>(bson) {
                            Ok(service_name) => {
                                // Check permissions
                                match Self::handle_auth_request(
                                    &service_name,
                                    &request,
                                    clients.clone(),
                                )
                                .await
                                {
                                    Ok(_) => {
                                        info!("Succesfully authorized {service_name}");
                                        request.respond(Ok(())).await;

                                        service_name
                                    }
                                    Err(e) => {
                                        warn!("Failed to register {service_name}");
                                        request.respond::<()>(Err(e)).await;
                                        request.writer().flush().await;

                                        return Err(());
                                    }
                                }
                            }
                            // Message deserialization error
                            Err(e) => {
                                warn!("Invalid Auth message body from a client: {e:?}");

                                request
                                    .respond::<()>(Err(Error::InternalError(e.to_string())))
                                    .await;
                                request.writer().flush().await;

                                return Err(());
                            }
                        }
                    }
                    // Not a call, but respond, of FD or other irrelevant message
                    _ => {
                        warn!("Invalid Auth message from a client (not a call)");
                        request
                            .respond::<()>(Err(Error::InternalError(
                                "Waiting for a registration message. Received a call".to_owned(),
                            )))
                            .await;
                        request.writer().flush().await;

                        return Err(());
                    }
                }
            }
            // Client disconnected
            _ => {
                return Err(());
            }
        };

        Ok((credentials.pid().unwrap(), service_name, rpc, log_sender))
    }

    fn client_name(status: std::result::Result<String, ()>) -> Option<String> {
        match status {
            Ok(service_name) => Some(service_name),
            _ => None,
        }
    }

    /// Handle client Auth message
    async fn handle_auth_request(
        service_name: &str,
        request: &RpcRequest,
        clients: ClientRegistryType,
    ) -> Result<()> {
        debug!("Service registration request: {}", service_name);

        let mut clients_lock = clients.lock().await;

        // Check if we already have a client with the name
        if clients_lock.contains_key(service_name) {
            warn!(
                "Multiple service registration request from: {}",
                service_name
            );

            Err(Error::AlreadyRegistered)
        // The only valid Auth request path
        } else {
            clients_lock.insert(service_name.to_owned(), request.writer().clone());

            info!("Client authorized as: {}", service_name);

            Ok(())
        }
    }
}
