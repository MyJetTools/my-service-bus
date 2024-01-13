use std::{sync::Arc, time::Duration};

use my_service_bus::abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{queues::DeliveryBucket, sessions::MyServiceBusSession};

use super::{SubscriberId, SubscriberMetrics};
#[derive(Debug)]
pub struct OnDeliveryStateData {
    pub bucket: DeliveryBucket,
    inserted: DateTimeAsMicroseconds,
}

#[derive(Debug)]
pub enum QueueSubscriberDeliveryState {
    ReadyToDeliver,
    Rented,
    OnDelivery(OnDeliveryStateData),
}

impl QueueSubscriberDeliveryState {
    pub fn to_string(&self) -> &str {
        match self {
            QueueSubscriberDeliveryState::ReadyToDeliver => "ReadyToDeliver",
            QueueSubscriberDeliveryState::Rented => "Rented",
            QueueSubscriberDeliveryState::OnDelivery(_) => "OnDelivery",
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            QueueSubscriberDeliveryState::ReadyToDeliver => 0,
            QueueSubscriberDeliveryState::Rented => 1,
            QueueSubscriberDeliveryState::OnDelivery(_) => 2,
        }
    }
}

pub struct QueueSubscriber {
    pub topic_id: String,
    pub queue_id: String,
    pub subscribed: DateTimeAsMicroseconds,
    pub metrics: SubscriberMetrics,
    pub delivery_state: QueueSubscriberDeliveryState,

    pub last_delivered: DateTimeAsMicroseconds,
    pub last_delivered_amount: usize,

    pub delivery_compilation_duration: Duration,

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
            delivery_state: QueueSubscriberDeliveryState::ReadyToDeliver,
            last_delivered: DateTimeAsMicroseconds::now(),
            session,
            id,
            last_delivered_amount: 0,
            delivery_compilation_duration: Duration::from_secs(0),
        }
    }

    pub fn rent_me(&mut self) -> bool {
        if let QueueSubscriberDeliveryState::ReadyToDeliver = &self.delivery_state {
            self.metrics.set_delivery_mode_as_rented();
            self.delivery_state = QueueSubscriberDeliveryState::Rented;
            return true;
        }

        return false;
    }

    pub fn get_on_delivery_amount(&self) -> usize {
        match &self.delivery_state {
            QueueSubscriberDeliveryState::ReadyToDeliver => 0,
            QueueSubscriberDeliveryState::Rented => 0,
            QueueSubscriberDeliveryState::OnDelivery(on_delivery) => {
                on_delivery.bucket.ids.queue_size()
            }
        }
    }

    pub fn cancel_the_rent(&mut self) {
        self.metrics.set_delivery_mode_as_ready_to_deliver();
        self.delivery_state = QueueSubscriberDeliveryState::ReadyToDeliver;
    }

    pub fn reset_delivery(&mut self) -> Option<DeliveryBucket> {
        self.last_delivered = DateTimeAsMicroseconds::now();
        let mut prev_delivery_state = QueueSubscriberDeliveryState::ReadyToDeliver;

        std::mem::swap(&mut prev_delivery_state, &mut self.delivery_state);

        self.metrics.set_delivery_mode_as_ready_to_deliver();
        if let QueueSubscriberDeliveryState::OnDelivery(state) = prev_delivery_state {
            self.last_delivered_amount = state.bucket.ids.queue_size();
            return Some(state.bucket);
        }

        return None;
    }

    pub fn intermediary_confirmed(&mut self, queue: &QueueWithIntervals) {
        if let QueueSubscriberDeliveryState::OnDelivery(state) = &mut self.delivery_state {
            state.bucket.confirmed(queue);
        }
    }

    pub fn set_messages_on_delivery(
        &mut self,
        messages: QueueWithIntervals,
        compilation_duration: Duration,
    ) {
        self.delivery_compilation_duration = compilation_duration;
        if let QueueSubscriberDeliveryState::Rented = &self.delivery_state {
            self.delivery_state = QueueSubscriberDeliveryState::OnDelivery(OnDeliveryStateData {
                bucket: DeliveryBucket::new(messages),
                inserted: DateTimeAsMicroseconds::now(),
            });
            self.metrics.set_delivery_mode_as_on_delivery();
            return;
        }

        panic!(
            "We are setting messages on delivery but previous state is '{}'. Previous state must be 'Rented'",
            self.delivery_state.to_string()
        );
    }

    pub fn get_messages_on_delivery(&self) -> Option<QueueWithIntervals> {
        match &self.delivery_state {
            QueueSubscriberDeliveryState::ReadyToDeliver => None,
            QueueSubscriberDeliveryState::Rented => None,
            QueueSubscriberDeliveryState::OnDelivery(state) => Some(state.bucket.ids.clone()),
        }
    }

    pub fn is_dead_on_delivery(&self, max_delivery_duration: Duration) -> Option<Duration> {
        match &self.delivery_state {
            QueueSubscriberDeliveryState::ReadyToDeliver => None,
            QueueSubscriberDeliveryState::Rented => None,
            QueueSubscriberDeliveryState::OnDelivery(state) => {
                let now = DateTimeAsMicroseconds::now();
                let duration = now.duration_since(state.inserted).as_positive_or_zero();
                if duration > max_delivery_duration {
                    return Some(duration);
                }

                return None;
            }
        }
    }

    pub fn get_min_message_id(&self) -> Option<MessageId> {
        let messages_on_delivery = self.get_messages_on_delivery()?;
        MessageId::from_opt_i64(messages_on_delivery.get_min_id())
    }

    //todo!("Check  who calls it")
    pub fn update_delivery_time(&mut self, amount: usize, positive: bool) {
        let delivery_duration = DateTimeAsMicroseconds::now()
            .duration_since(self.metrics.start_delivery_time)
            .as_positive_or_zero();

        if delivery_duration.is_zero() {
            println!(
                "Delivery duration is zero. This is a bug. Please report it. (update_delivery_time)"
            )
        }

        if positive {
            self.metrics
                .set_delivered_statistic(amount, delivery_duration);
        } else {
            self.metrics
                .set_not_delivered_statistic(amount as i32, delivery_duration);
        }
    }
}
