use std::sync::Arc;

use my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus_tcp_shared::{MySbTcpSerializer, TcpContract};
use my_tcp_sockets::tcp_connection::SocketConnection;

use crate::{app::AppContext, operations};

use super::error::MySbSocketError;

pub async fn handle(
    app: &Arc<AppContext>,
    tcp_contract: TcpContract,
    connection: Arc<SocketConnection<TcpContract, MySbTcpSerializer>>,
) -> Result<(), MySbSocketError> {
    match tcp_contract {
        TcpContract::Ping {} => {
            connection.send(TcpContract::Pong).await;
            Ok(())
        }
        TcpContract::Pong {} => Ok(()),
        TcpContract::Greeting {
            name,
            protocol_version,
        } => {
            println!("Greeting: {}. Protocol: {}", name, protocol_version);
            let splitted: Vec<&str> = name.split(";").collect();

            if let Some(session) = app.sessions.get_by_tcp_connection_id(connection.id).await {
                if splitted.len() == 2 {
                    session
                        .set_tcp_socket_name(splitted[0].to_string(), Some(splitted[1].to_string()))
                        .await;
                } else {
                    session.set_tcp_socket_name(name, None).await;
                }

                session.update_tcp_protocol_version(protocol_version);
            }

            Ok(())
        }
        TcpContract::Publish {
            topic_id,
            request_id,
            persist_immediately,
            data_to_publish,
        } => {
            println!("Publish: {}.", topic_id);
            if let Some(session_id) = app
                .sessions
                .resolve_session_id_by_tcp_connection_id(connection.id)
                .await
            {
                let result = operations::publisher::publish(
                    app,
                    topic_id.as_str(),
                    data_to_publish,
                    persist_immediately,
                    session_id,
                )
                .await;

                if let Err(err) = result {
                    connection
                        .send(TcpContract::Reject {
                            message: format!("{:?}", err),
                        })
                        .await;
                } else {
                    connection
                        .send(TcpContract::PublishResponse { request_id })
                        .await;
                }
            }

            Ok(())
        }

        TcpContract::PublishResponse { request_id: _ } => {
            //This is a client packet
            Ok(())
        }
        TcpContract::Subscribe {
            topic_id,
            queue_id,
            queue_type,
        } => {
            println!("Subscribe: {}/{}.", topic_id, queue_id);
            if let Some(session) = app.sessions.get_by_tcp_connection_id(connection.id).await {
                operations::subscriber::subscribe_to_queue(
                    app, topic_id, queue_id, queue_type, &session,
                )
                .await?;
            }

            Ok(())
        }
        TcpContract::SubscribeResponse {
            topic_id: _,
            queue_id: _,
        } => {
            //This is a client packet
            Ok(())
        }
        TcpContract::Raw(_) => {
            //This is a client packet
            Ok(())
        }
        TcpContract::NewMessagesConfirmation {
            topic_id,
            queue_id,
            confirmation_id,
        } => {
            println!("Confirm: {}/{}.", topic_id, queue_id);
            operations::delivery_confirmation::all_confirmed(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                confirmation_id.into(),
            )
            .await?;
            Ok(())
        }
        TcpContract::CreateTopicIfNotExists { topic_id } => {
            if let Some(session) = app.sessions.get_by_tcp_connection_id(connection.id).await {
                operations::publisher::create_topic_if_not_exists(
                    app,
                    Some(session.id),
                    topic_id.as_str(),
                )
                .await?;
            }

            Ok(())
        }
        TcpContract::IntermediaryConfirm {
            packet_version: _,
            topic_id,
            queue_id,
            confirmation_id,
            delivered,
        } => {
            operations::delivery_confirmation::intermediary_confirm(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                confirmation_id.into(),
                QueueWithIntervals::restore(delivered),
            )
            .await?;

            Ok(())
        }
        TcpContract::PacketVersions { packet_versions } => {
            if let Some(version) =
                packet_versions.get(&my_service_bus_tcp_shared::tcp_message_id::NEW_MESSAGES)
            {
                if let Some(session) = app.sessions.get_by_tcp_connection_id(connection.id).await {
                    session.update_tcp_delivery_packet_version(*version)
                }
            }

            Ok(())
        }
        TcpContract::Reject { message: _ } => {
            //This is a client packet
            Ok(())
        }
        TcpContract::AllMessagesConfirmedAsFail {
            topic_id,
            queue_id,
            confirmation_id,
        } => {
            operations::delivery_confirmation::all_fail(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                confirmation_id.into(),
            )
            .await?;
            Ok(())
        }

        TcpContract::ConfirmSomeMessagesAsOk {
            packet_version: _,
            topic_id,
            queue_id,
            confirmation_id,
            delivered,
        } => {
            operations::delivery_confirmation::some_messages_are_confirmed(
                app,
                topic_id.as_str(),
                queue_id.as_str(),
                confirmation_id.into(),
                QueueWithIntervals::restore(delivered),
            )
            .await?;

            Ok(())
        }
        TcpContract::NewMessages {
            topic_id: _,
            queue_id: _,
            confirmation_id: _,
            messages: _,
        } => {
            //this is Client Side Message

            Ok(())
        }
    }
}
