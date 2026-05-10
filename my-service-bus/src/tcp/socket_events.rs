use async_trait::async_trait;
use my_logger::LogEventCtx;
use my_tcp_sockets::SocketEventCallback;
use std::sync::Arc;

use my_service_bus::{
    abstractions::queue_with_intervals::QueueWithIntervals,
    tcp_contracts::{MySbSerializerState, MySbTcpConnection, MySbTcpContract, MySbTcpSerializer},
};

use crate::{app::AppContext, operations};

use super::error::MySbSocketError;

#[derive(Clone)]
pub struct TcpServerEvents {
    app: Arc<AppContext>,
}

impl TcpServerEvents {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }

    pub async fn handle_incoming_packet(
        &self,
        tcp_contract: MySbTcpContract,
        connection: &Arc<MySbTcpConnection>,
    ) -> Result<(), MySbSocketError> {
        match tcp_contract {
            MySbTcpContract::Ping {} => {
                connection.send(&MySbTcpContract::Pong);
                Ok(())
            }
            MySbTcpContract::Pong {} => Ok(()),
            MySbTcpContract::Greeting {
                name,
                protocol_version,
            } => {
                println!(
                    "New tcp connection [{}] with name: {} and protocol_version {}",
                    connection.id, name, protocol_version
                );
                let mut connection_name = None;
                let mut version = None;
                let mut env_info = None;

                let mut no = 0;
                for itm in name.split(";") {
                    match no {
                        0 => connection_name = Some(itm.to_string()),
                        1 => version = Some(itm.to_string()),
                        2 => env_info = Some(itm.to_string()),
                        _ => {}
                    }
                    no += 1;
                }

                self.app.sessions.add_tcp(
                    connection.clone(),
                    connection_name.unwrap(),
                    version,
                    env_info,
                    protocol_version,
                );

                Ok(())
            }
            MySbTcpContract::Publish {
                topic_id,
                request_id,
                persist_immediately,
                data_to_publish,
            } => {
                if let Some(session_id) = self
                    .app
                    .sessions
                    .get_session_id_by_tcp_connection_id(connection.id)
                {
                    let result = operations::publisher::publish(
                        &self.app,
                        topic_id.as_str(),
                        data_to_publish,
                        persist_immediately,
                        session_id,
                    ).await;

                    if let Err(err) = result {
                        connection
                            .send(&MySbTcpContract::Reject {
                                message: format!("{:?}", err),
                            });
                    } else {
                        connection
                            .send(&MySbTcpContract::PublishResponse { request_id });
                    }
                }

                Ok(())
            }

            MySbTcpContract::PublishResponse { request_id: _ } => {
                //This is a client packet
                Ok(())
            }
            MySbTcpContract::Subscribe {
                topic_id,
                queue_id,
                queue_type,
            } => {
                println!(
                    "Subscribe packet received: connection_id={} topic_id={} queue_id={} queue_type={:?}",
                    connection.id, topic_id, queue_id, queue_type
                );
                if let Some(session) = self
                    .app
                    .sessions
                    .get_tcp_session_by_connection_id(connection.id)
                {
                    operations::subscriber::subscribe_to_queue(
                        &self.app,
                        topic_id,
                        queue_id,
                        queue_type,
                        session.into(),
                    )
                    .await?;
                } else {
                    println!(
                        "Subscribe packet ignored: no session for connection_id={}",
                        connection.id
                    );
                }

                Ok(())
            }
            MySbTcpContract::SubscribeResponse {
                topic_id: _,
                queue_id: _,
            } => {
                //This is a client packet
                Ok(())
            }
            MySbTcpContract::Raw(_) => {
                //This is a client packet
                Ok(())
            }
            MySbTcpContract::NewMessagesConfirmation {
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                operations::delivery_confirmation::all_confirmed(
                    &self.app,
                    topic_id.as_str(),
                    queue_id.as_str(),
                    confirmation_id.into(),
                )
                .await?;

                Ok(())
            }
            MySbTcpContract::CreateTopicIfNotExists { topic_id } => {
                if let Some(session) = self
                    .app
                    .sessions
                    .get_session_id_by_tcp_connection_id(connection.id)
                {
                    operations::create_topic_if_not_exists(
                        &self.app,
                        Some(session),
                        topic_id.as_str(),
                    )
                    .await?;
                }

                Ok(())
            }
            MySbTcpContract::IntermediaryConfirm {
                packet_version: _,
                topic_id,
                queue_id,
                confirmation_id,
                delivered,
            } => {
                operations::delivery_confirmation::intermediary_confirm(
                    &self.app,
                    topic_id.as_str(),
                    queue_id.as_str(),
                    confirmation_id.into(),
                    QueueWithIntervals::restore(delivered),
                )
                .await?;

                Ok(())
            }
            MySbTcpContract::PacketVersions { packet_versions } => {
                if let Some(version) = packet_versions
                    .get(&my_service_bus::tcp_contracts::tcp_message_id::NEW_MESSAGES)
                {
                    if let Some(session) = &self
                        .app
                        .sessions
                        .get_tcp_session_by_connection_id(connection.id)
                    {
                        session.update_deliver_message_packet_version(*version as u8)
                    }
                }

                Ok(())
            }
            MySbTcpContract::Reject { message: _ } => {
                //This is a client packet
                Ok(())
            }
            MySbTcpContract::AllMessagesConfirmedAsFail {
                topic_id,
                queue_id,
                confirmation_id,
            } => {
                operations::delivery_confirmation::all_fail(
                    &self.app,
                    topic_id.as_str(),
                    queue_id.as_str(),
                    confirmation_id.into(),
                )
                .await?;
                Ok(())
            }

            MySbTcpContract::ConfirmSomeMessagesAsOk {
                packet_version: _,
                topic_id,
                queue_id,
                confirmation_id,
                delivered,
            } => {
                operations::delivery_confirmation::some_messages_are_confirmed(
                    &self.app,
                    topic_id.as_str(),
                    queue_id.as_str(),
                    confirmation_id.into(),
                    QueueWithIntervals::restore(delivered),
                )
                .await?;

                Ok(())
            }
            MySbTcpContract::NewMessages(_) => {
                //this is Client Side Message

                Ok(())
            }
        }
    }
}

#[async_trait]
impl SocketEventCallback<MySbTcpContract, MySbTcpSerializer, MySbSerializerState>
    for TcpServerEvents
{
    async fn connected(&mut self, _connection: Arc<MySbTcpConnection>) {
        self.app.prometheus.mark_new_tcp_connection();
    }

    async fn disconnected(&mut self, connection: Arc<MySbTcpConnection>) {
        self.app.prometheus.mark_new_tcp_disconnection();
        if let Some(session) = self.app.sessions.remove_tcp(connection.id) {
            crate::operations::sessions::disconnect(self.app.as_ref(), session).await;
        }
    }

    async fn payload(&mut self, connection: &Arc<MySbTcpConnection>, contract: MySbTcpContract) {
        let connection_id = connection.id;
        if let Err(err) = self.handle_incoming_packet(contract, connection).await {
            my_logger::LOGGER.write_error(
                "Handle Tcp Payload".to_string(),
                format!("{:?}", err),
                LogEventCtx::new().add("connectionId", connection_id.to_string()),
            );
        }
    }
}
