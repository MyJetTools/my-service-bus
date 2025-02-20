use std::{sync::Arc, time::Duration};

use my_service_bus::abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use rust_extensions::{date_time::DateTimeAsMicroseconds, sorted_vec::EntityWithKey, StrOrString};

use crate::{
    queues::{DeliveryBucket, QueueId},
    sessions::MyServiceBusSession,
};

use super::{SubscriberId, SubscriberMetrics};
#[derive(Debug)]
pub struct OnDeliveryStateData {
    pub bucket: DeliveryBucket,
    last_update: DateTimeAsMicroseconds,
}

#[derive(Debug)]
pub enum QueueSubscriberDeliveryState {
    Idle,
    Rented,
    OnDelivery(OnDeliveryStateData),
}

impl QueueSubscriberDeliveryState {
    pub fn as_str(&self) -> StrOrString<'static> {
        match self {
            QueueSubscriberDeliveryState::Idle => "Idle".into(),
            QueueSubscriberDeliveryState::Rented => "Rented".into(),
            QueueSubscriberDeliveryState::OnDelivery(data) => {
                let now = DateTimeAsMicroseconds::now();

                let duration = now - data.last_update;
                format!("{:?} {:?}", duration, data.bucket.ids.len()).into()
            }
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            QueueSubscriberDeliveryState::Idle => 0,
            QueueSubscriberDeliveryState::Rented => 1,
            QueueSubscriberDeliveryState::OnDelivery(_) => 2,
        }
    }
}

pub struct QueueSubscriber {
    pub queue_id: QueueId,
    pub subscribed: DateTimeAsMicroseconds,
    pub metrics: SubscriberMetrics,
    pub delivery_state: QueueSubscriberDeliveryState,

    pub last_delivered: DateTimeAsMicroseconds,
    pub last_delivered_amount: usize,

    pub delivery_compilation_duration: Duration,

    pub id: SubscriberId,
    pub session: Arc<dyn MyServiceBusSession + Send + Sync + 'static>,
}

impl QueueSubscriber {
    pub fn new(
        id: SubscriberId,
        queue_id: QueueId,
        session: Arc<dyn MyServiceBusSession + Send + Sync + 'static>,
    ) -> Self {
        Self {
            queue_id: queue_id.clone(),
            subscribed: DateTimeAsMicroseconds::now(),
            metrics: SubscriberMetrics::new(),
            delivery_state: QueueSubscriberDeliveryState::Idle,
            last_delivered: DateTimeAsMicroseconds::now(),
            session,
            id,
            last_delivered_amount: 0,
            delivery_compilation_duration: Duration::from_secs(0),
        }
    }

    pub fn rent_me(&mut self) -> bool {
        if let QueueSubscriberDeliveryState::Idle = &self.delivery_state {
            self.metrics.set_delivery_mode_as_rented();
            self.delivery_state = QueueSubscriberDeliveryState::Rented;
            return true;
        }

        return false;
    }

    pub fn get_on_delivery_amount(&self) -> usize {
        match &self.delivery_state {
            QueueSubscriberDeliveryState::Idle => 0,
            QueueSubscriberDeliveryState::Rented => 0,
            QueueSubscriberDeliveryState::OnDelivery(on_delivery) => {
                on_delivery.bucket.ids.queue_size()
            }
        }
    }

    pub fn cancel_the_rent(&mut self) {
        println!("Cancel the rent");
        self.metrics.set_delivery_mode_as_ready_to_deliver();
        self.delivery_state = QueueSubscriberDeliveryState::Idle;
    }

    pub fn reset_delivery(&mut self) -> Option<DeliveryBucket> {
        self.last_delivered = DateTimeAsMicroseconds::now();
        let mut prev_delivery_state = QueueSubscriberDeliveryState::Idle;

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
            state.last_update = DateTimeAsMicroseconds::now();
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
                last_update: DateTimeAsMicroseconds::now(),
            });
            self.metrics.set_delivery_mode_as_on_delivery();

            return;
        }

        panic!(
            "We are setting messages on delivery but previous state is '{}'. Previous state must be 'Rented'",
            self.delivery_state.as_str()
        );
    }

    pub fn get_messages_on_delivery(&self) -> Option<QueueWithIntervals> {
        match &self.delivery_state {
            QueueSubscriberDeliveryState::Idle => None,
            QueueSubscriberDeliveryState::Rented => None,
            QueueSubscriberDeliveryState::OnDelivery(state) => Some(state.bucket.ids.clone()),
        }
    }

    pub fn get_messages_amount_on_delivery(&self) -> usize {
        match &self.delivery_state {
            QueueSubscriberDeliveryState::Idle => 0,
            QueueSubscriberDeliveryState::Rented => 0,
            QueueSubscriberDeliveryState::OnDelivery(state) => state.bucket.ids.queue_size(),
        }
    }

    pub fn is_dead_on_delivery(&self, max_delivery_duration: Duration) -> Option<Duration> {
        match &self.delivery_state {
            QueueSubscriberDeliveryState::Idle => None,
            QueueSubscriberDeliveryState::Rented => None,
            QueueSubscriberDeliveryState::OnDelivery(state) => {
                let now = DateTimeAsMicroseconds::now();
                let duration = now.duration_since(state.last_update).as_positive_or_zero();
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

    //todo!("TechDebt: We does not call it with intermediary confirmed messages");
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

impl EntityWithKey<i64> for QueueSubscriber {
    fn get_key(&self) -> &i64 {
        &self.id
    }
}
