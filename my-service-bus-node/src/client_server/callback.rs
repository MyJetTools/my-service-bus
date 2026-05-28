use std::sync::Arc;

use async_trait::async_trait;
use my_logger::LogEventCtx;
use my_tcp_sockets::SocketEventCallback;

use my_service_bus::tcp_contracts::{
    MySbSerializerState, MySbTcpConnection, MySbTcpContract, MySbTcpSerializer,
};

use crate::app::AppContext;

#[derive(Clone)]
pub struct ClientServerCallback {
    app: Arc<AppContext>,
}

impl ClientServerCallback {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl SocketEventCallback<MySbTcpContract, MySbTcpSerializer, MySbSerializerState>
    for ClientServerCallback
{
    async fn connected(&mut self, connection: Arc<MySbTcpConnection>) {
        self.app.client_sessions.add(connection).await;
    }

    async fn disconnected(&mut self, connection: Arc<MySbTcpConnection>) {
        self.app.client_sessions.remove(connection.id).await;
    }

    async fn payload(&mut self, connection: &Arc<MySbTcpConnection>, contract: MySbTcpContract) {
        match contract {
            MySbTcpContract::Ping => {
                connection.send(&MySbTcpContract::Pong);
            }
            MySbTcpContract::Pong => {}
            MySbTcpContract::Greeting {
                name: _,
                protocol_version: _,
            } => {
                // We already track the connection on `connected()`. Nothing
                // else to do here for now.
            }
            MySbTcpContract::PacketVersions { packet_versions: _ } => {
                // No-op for node — packet version negotiation only matters
                // for the master delivery path, which we don't relay yet.
            }
            MySbTcpContract::Publish {
                topic_id,
                request_id,
                persist_immediately,
                data_to_publish,
            } => {
                self.app
                    .outbound
                    .accept_client_publish(
                        topic_id,
                        persist_immediately,
                        data_to_publish,
                        connection.id,
                        request_id,
                    )
                    .await;
            }
            MySbTcpContract::CreateTopicIfNotExists { topic_id } => {
                // Forward as-is to master if connected; otherwise drop silently
                // — clients usually do this opportunistically.
                if let Some(master) = self.app.get_master_connection().await {
                    master.send(&MySbTcpContract::CreateTopicIfNotExists { topic_id });
                }
            }
            other => {
                my_logger::LOGGER.write_warning(
                    "client_unsupported_packet",
                    format!(
                        "Client sent {} — subscribe path not implemented yet",
                        other.as_str()
                    ),
                    LogEventCtx::new().add("connection_id", connection.id.to_string()),
                );
            }
        }
    }
}
