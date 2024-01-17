use async_trait::async_trait;
use my_logger::LogEventCtx;
use my_tcp_sockets::SocketEventCallback;
use std::sync::Arc;

use my_service_bus::tcp_contracts::{
    MySbSerializerState, MySbTcpConnection, MySbTcpContract, MySbTcpSerializer,
};

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
impl SocketEventCallback<MySbTcpContract, MySbTcpSerializer, MySbSerializerState>
    for TcpServerEvents
{
    async fn connected(&self, _connection: Arc<MySbTcpConnection>) {
        self.app.prometheus.mark_new_tcp_connection();
    }

    async fn disconnected(&self, connection: Arc<MySbTcpConnection>) {
        self.app.prometheus.mark_new_tcp_disconnection();
        if let Some(session) = self.app.sessions.remove_tcp(connection.id).await {
            crate::operations::sessions::disconnect(self.app.as_ref(), session.as_ref()).await;
        }
    }

    async fn payload(&self, connection: &Arc<MySbTcpConnection>, contract: MySbTcpContract) {
        let connection_id = connection.id;
        if let Err(err) = super::incoming_packets::handle(&self.app, contract, connection).await {
            my_logger::LOGGER.write_error(
                "Handle Tcp Payload".to_string(),
                format!("{:?}", err),
                LogEventCtx::new().add("connectionId", connection_id.to_string()),
            );
        }
    }
}
