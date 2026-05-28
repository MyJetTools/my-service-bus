use std::sync::Arc;

use async_trait::async_trait;
use my_logger::LogEventCtx;
use my_tcp_sockets::SocketEventCallback;

use my_service_bus::tcp_contracts::{
    MySbSerializerState, MySbTcpConnection, MySbTcpContract, MySbTcpSerializer,
};

use crate::{app::AppContext, outbound::Ack};

/// Protocol version exchanged in `NodeGreeting`. Node packets are only valid
/// at protocol v3+ so we always announce 3.
const NODE_PROTOCOL_VERSION: i32 = 3;

#[derive(Clone)]
pub struct MasterClientCallback {
    app: Arc<AppContext>,
}

impl MasterClientCallback {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    async fn handle_node_publish_response(
        &self,
        sequence_number: i64,
        connection: &Arc<MySbTcpConnection>,
    ) {
        let acks = match self.app.outbound.complete_in_flight(sequence_number).await {
            Ok(acks) => acks,
            Err(expected) => {
                my_logger::LOGGER.write_error(
                    "node_publish_response",
                    format!(
                        "sequence mismatch: response={} expected={}, dropping master connection",
                        sequence_number, expected
                    ),
                    LogEventCtx::new(),
                );
                connection.disconnect().await;
                return;
            }
        };
        self.fan_out_publish_acks(acks).await;
    }

    async fn fan_out_publish_acks(&self, acks: Vec<Ack>) {
        for ack in acks {
            if let Some(client_conn) = self.app.client_sessions.get(ack.client_connection_id).await
            {
                client_conn.send(&MySbTcpContract::PublishResponse {
                    request_id: ack.client_request_id,
                });
            }
        }
    }
}

#[async_trait]
impl SocketEventCallback<MySbTcpContract, MySbTcpSerializer, MySbSerializerState>
    for MasterClientCallback
{
    async fn connected(&mut self, connection: Arc<MySbTcpConnection>) {
        let hostname = std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".to_string());
        let name = format!("node:{};0.1.0", hostname);

        // Order matters: store the connection BEFORE sending the greeting so
        // the flusher (if it wakes up immediately after) can use it.
        self.app
            .set_master_connection(Some(connection.clone()))
            .await;

        connection.send(&MySbTcpContract::NodeGreeting {
            name,
            protocol_version: NODE_PROTOCOL_VERSION,
        });

        // Kick the flusher in case there's already buffered data waiting from
        // a previous (now-stale) master connection.
        self.app.outbound.pulse();
    }

    async fn disconnected(&mut self, _connection: Arc<MySbTcpConnection>) {
        self.app.set_master_connection(None).await;

        // Everything buffered or in-flight is lost (at-most-once semantics).
        // Reject the corresponding clients so they don't hang waiting for ack.
        let stale_acks = self.app.outbound.drain_all_on_disconnect().await;
        for ack in stale_acks {
            if let Some(client_conn) = self.app.client_sessions.get(ack.client_connection_id).await
            {
                client_conn.send(&MySbTcpContract::Reject {
                    message: "master connection lost".to_string(),
                });
            }
        }
    }

    async fn payload(&mut self, connection: &Arc<MySbTcpConnection>, contract: MySbTcpContract) {
        match contract {
            MySbTcpContract::Ping => {
                connection.send(&MySbTcpContract::Pong);
            }
            MySbTcpContract::Pong => {}
            MySbTcpContract::NodePublishResponse {
                sequence_number,
                topics: _,
            } => {
                self.handle_node_publish_response(sequence_number, connection)
                    .await;
            }
            MySbTcpContract::Reject { message } => {
                my_logger::LOGGER.write_error(
                    "master_reject",
                    format!("master rejected node packet: {}", message),
                    LogEventCtx::new(),
                );
                // Conservative: close so we drain pending acks and reset state.
                connection.disconnect().await;
            }
            other => {
                // Subscribe path not implemented yet; anything else is unexpected.
                my_logger::LOGGER.write_warning(
                    "master_unexpected_packet",
                    format!("unhandled packet from master: {}", other.as_str()),
                    LogEventCtx::new(),
                );
            }
        }
    }
}
