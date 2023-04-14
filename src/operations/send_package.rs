use std::sync::Arc;

use crate::{sessions::MyServiceBusSession, topics::TopicData};

use super::delivery::{SendNewMessagesResult, SubscriberPackageBuilder};

pub fn send_package(
    session: Arc<MyServiceBusSession>,
    tcp_packet: my_service_bus_tcp_shared::TcpContract,
) {
    let _handle = tokio::spawn(async move {
        match &session.connection {
            crate::sessions::SessionConnection::Tcp(data) => {
                data.connection.send(tcp_packet).await;
            }
            #[cfg(test)]
            crate::sessions::SessionConnection::Test(data) => {
                data.send_packet(tcp_packet).await;
            }
            crate::sessions::SessionConnection::Http(_) => todo!("Not suppored yet"),
        }
    });
}

pub fn send_new_messages_to_deliver(builder: SubscriberPackageBuilder, topic_data: &mut TopicData) {
    let subscriber_id = builder.subscriber_id;

    match builder.get_result() {
        SendNewMessagesResult::Send {
            session,
            tcp_contract,
            queue_id,
            messages_on_delivery,
        } => {
            if let Some(queue) = topic_data.queues.get_mut(queue_id.as_str()) {
                if let Some(subscriber) = queue.subscribers.get_by_id_mut(subscriber_id) {
                    subscriber.set_messages_on_delivery(messages_on_delivery);
                    send_package(session, tcp_contract);
                    subscriber.metrics.set_started_delivery();
                }
            }
        }
        SendNewMessagesResult::NothingToSend { queue_id } => {
            if let Some(queue) = topic_data.queues.get_mut(queue_id.as_str()) {
                if let Some(subscriber) = queue.subscribers.get_by_id_mut(subscriber_id) {
                    subscriber.cancel_the_rent();
                }
            }
        }
    }
}
