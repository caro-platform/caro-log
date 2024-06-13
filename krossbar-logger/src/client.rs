use futures::{channel::mpsc::Sender, SinkExt};
use log::{trace, warn};
use tokio::net::unix;

use krossbar_rpc::{rpc::Rpc, Error};

use krossbar_log_common::{log_message::LogMessage, LOGGING_METHOD_NAME};

use crate::LogEvent;

pub struct Client {
    pid: unix::pid_t,
    service_name: String,
    rpc: Rpc,
    log_sender: Sender<LogEvent>,
}

impl Client {
    pub async fn run(
        (pid, service_name, rpc, log_sender): (unix::pid_t, String, Rpc, Sender<LogEvent>),
    ) -> std::result::Result<String, ()> {
        let this = Self {
            pid,
            rpc,
            service_name: service_name.clone(),
            log_sender,
        };

        this.client_loop().await;

        Ok(service_name)
    }

    pub async fn client_loop(mut self) -> String {
        loop {
            match self.rpc.poll().await {
                Some(mut request) => {
                    if request.endpoint() != LOGGING_METHOD_NAME {
                        request
                            .respond::<()>(Err(Error::InternalError(format!(
                                "Expected log from a client. Got {}",
                                request.endpoint()
                            ))))
                            .await;
                    }

                    match request.take_body().unwrap() {
                        // Valid one-way message
                        krossbar_rpc::request::Body::Message(bson) => {
                            // Valid Auth message
                            match bson::from_bson::<LogMessage>(bson) {
                                Ok(log_message) => {
                                    self.handle_log_message(log_message).await;
                                }
                                // Message deserialization error
                                Err(e) => {
                                    warn!("Invalid connection message body from a client: {e:?}");

                                    request
                                        .respond::<()>(Err(Error::InternalError(e.to_string())))
                                        .await;
                                    return self.service_name;
                                }
                            }
                        }
                        // Not a call, but respond, of FD or other irrelevant message
                        _ => {
                            warn!("Invalid connection message from a client (not a call)");
                            return self.service_name;
                        }
                    }
                }
                _ => return self.service_name,
            }
        }
    }

    async fn handle_log_message(&mut self, message: LogMessage) {
        trace!(
            "Incoming log message from {}: {message:?}",
            self.service_name
        );

        let _ = self
            .log_sender
            .send(LogEvent {
                pid: self.pid,
                service_name: self.service_name.clone(),
                message,
            })
            .await;
    }
}
