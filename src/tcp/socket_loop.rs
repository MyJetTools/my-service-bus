use async_trait::async_trait;
use my_tcp_sockets::{ConnectionEvent, SocketEventCallback};
use std::{collections::HashMap, sync::Arc};

use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};

use crate::{
    app::{logs::SystemProcess, AppContext},
    sessions::TcpConnectionData,
};

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
            ConnectionEvent::Connected(connection) => {
                println!("New tcp connection: {}", connection.id);

                self.app
                    .sessions
                    .add_tcp(TcpConnectionData::new(connection))
                    .await;
            }
            ConnectionEvent::Disconnected(connection) => {
                println!("Connection {} is disconnected", connection.id);
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
                    let mut ctx = HashMap::new();
                    ctx.insert("ConnectionId".to_string(), connection_id.to_string());

                    crate::LOGS.add_error(
                        None,
                        SystemProcess::TcpSocket,
                        "Handle Payload".to_string(),
                        format!("Err: {:?}", err),
                        Some(ctx),
                    );
                }
            }
        }
    }
}
