use std::{sync::Arc, time::Duration};

use my_service_bus_abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus_shared::MySbMessageContent;
use my_service_bus_tcp_shared::delivery_package_builder::DeliverTcpPacketBuilder;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{queues::DeliveryBucket, sessions::MyServiceBusSession};

use super::{SubscriberId, SubscriberMetrics};

pub struct OnDeliveryStateData {
    pub bucket: DeliveryBucket,
    inserted: DateTimeAsMicroseconds,
}

pub struct QueueSubscriber {
    pub topic_id: String,
    pub queue_id: String,
    pub subscribed: DateTimeAsMicroseconds,
    pub metrics: SubscriberMetrics,
    pub on_delivery: Option<OnDeliveryStateData>,

    pub id: SubscriberId,
    pub session: Arc<MyServiceBusSession>,
}

impl QueueSubscriber {
    pub fn new(
        id: SubscriberId,
        topic_id: String,
        queue_id: String,
        session: Arc<MyServiceBusSession>,
    ) -> Self {
        Self {
            topic_id: topic_id.to_string(),
            queue_id: queue_id.to_string(),
            subscribed: DateTimeAsMicroseconds::now(),
            metrics: SubscriberMetrics::new(id, session.id, topic_id, queue_id),
            on_delivery: None,
            session,
            id,
        }
    }

    pub fn get_on_delivery_amount(&self) -> i64 {
        match &self.on_delivery {
            Some(on_delivery) => on_delivery.bucket.ids.len(),
            None => 0,
        }
    }

    pub fn reset_delivery(&mut self) -> Option<DeliveryBucket> {
        let mut prev_delivery_state = None;
        std::mem::swap(&mut prev_delivery_state, &mut self.on_delivery);

        self.metrics.set_delivery_mode_as_ready_to_deliver();
        if let Some(state) = prev_delivery_state {
            return Some(state.bucket);
        }

        return None;
    }

    pub fn intermediary_confirmed(&mut self, queue: &QueueWithIntervals) {
        if let Some(state) = &mut self.on_delivery {
            state.bucket.confirmed(queue);
        }
    }

    fn set_messages_on_delivery(&mut self, messages: QueueWithIntervals) {
        if self.on_delivery.is_some() {
            panic!("Somehow we are trying to deliver to subscriber which already has messages on delivery");
        }

        let delivery_state = OnDeliveryStateData {
            bucket: DeliveryBucket::new(messages),
            inserted: DateTimeAsMicroseconds::now(),
        };

        self.on_delivery = Some(delivery_state);
    }

    pub fn get_messages_on_delivery(&self) -> Option<QueueWithIntervals> {
        let on_delivery = self.on_delivery.as_ref()?;
        Some(on_delivery.bucket.ids.clone())
    }

    pub fn is_dead_on_delivery(&self, max_delivery_duration: Duration) -> Option<Duration> {
        let on_delivery = self.on_delivery.as_ref()?;
        let now = DateTimeAsMicroseconds::now();
        let duration = now
            .duration_since(on_delivery.inserted)
            .as_positive_or_zero();
        if duration > max_delivery_duration {
            return Some(duration);
        }

        None
    }

    pub fn get_min_message_id(&self) -> Option<MessageId> {
        let messages_on_delivery = self.get_messages_on_delivery()?;
        return messages_on_delivery.get_min_id();
    }

    pub async fn deliver_messages(&mut self, messages: Vec<(i32, &MySbMessageContent)>) {
        self.metrics.set_started_delivery();
        match &self.session.connection {
            crate::sessions::SessionConnection::Tcp(connection) => {
                let mut payload = DeliverTcpPacketBuilder::new(
                    &self.topic_id,
                    &self.queue_id,
                    self.id,
                    connection.get_messages_to_deliver_protocol_version(),
                );

                let mut messages_on_delivery = QueueWithIntervals::new();

                for (attempt_no, msg) in messages {
                    payload.append_packet(msg, attempt_no);
                    messages_on_delivery.enqueue(msg.id);
                }

                let tcp_packet = payload.get_result();

                connection.connection.send(tcp_packet).await;

                self.set_messages_on_delivery(messages_on_delivery);
            }
            crate::sessions::SessionConnection::Http(_) => {
                todo!("Http connection is not supported for delivery");
            }
            #[cfg(test)]
            crate::sessions::SessionConnection::Test(connection) => {
                let mut messages_on_delivery = QueueWithIntervals::new();

                for (_, msg) in &messages {
                    messages_on_delivery.enqueue(msg.id);
                }

                connection.deliver_messages(messages).await;
                self.set_messages_on_delivery(messages_on_delivery);
            }
        }
    }
}
