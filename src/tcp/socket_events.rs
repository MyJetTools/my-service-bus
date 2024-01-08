use async_trait::async_trait;
use my_logger::LogEventCtx;
use my_tcp_sockets::{ConnectionEvent, SocketEventCallback};
use std::sync::Arc;

use my_service_bus::tcp_contracts::{MySbTcpSerializer, TcpContract};

use crate::app::AppContext;

pub struct TcpServerEvents {
    app: Arc<AppContext>,
}

impl TcpServerEvents {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl SocketEventCallback<TcpContract, MySbTcpSerializer> for TcpServerEvents {
    async fn handle(&self, connection_event: ConnectionEvent<TcpContract, MySbTcpSerializer>) {
        match connection_event {
            ConnectionEvent::Connected(_) => {
                self.app.prometheus.mark_new_tcp_connection();
            }
            ConnectionEvent::Disconnected(connection) => {
                self.app.prometheus.mark_new_tcp_disconnection();
                if let Some(session) = self.app.sessions.remove_tcp(connection.id).await {
                    crate::operations::sessions::disconnect(self.app.as_ref(), session.as_ref())
                        .await;
                }
            }
            ConnectionEvent::Payload {
                connection,
                payload,
            } => {
                let connection_id = connection.id;
                if let Err(err) =
                    super::incoming_packets::handle(&self.app, payload, connection).await
                {
                    my_logger::LOGGER.write_error(
                        "Handle Tcp Payload".to_string(),
                        format!("{:?}", err),
                        LogEventCtx::new().add("connectionId", connection_id.to_string()),
                    );
                }
            }
        }
    }
}
