use std::time::Duration;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    metric_data::{MetricOneSecond, MetricsHistory},
    sessions::SessionId,
};

use super::SubscriberId;

pub const DELIVERY_STATE_READY_TO_DELIVER: u8 = 0;
pub const DELIVERY_STATE_RENTED: u8 = 1;
pub const DELIVERY_STATE_ON_DELIVERY: u8 = 2;

#[derive(Clone)]
pub struct SubscriberMetrics {
    pub topic_id: String,
    pub queue_id: String,
    pub start_delivery_time: DateTimeAsMicroseconds,
    pub delivered_amount: MetricOneSecond,
    pub delivery_microseconds: MetricOneSecond,
    pub active: u8,
    pub delivery_history: MetricsHistory,

    pub session_id: SessionId,
    pub subscriber_id: SubscriberId,

    pub delivery_mode: u8,
}

impl SubscriberMetrics {
    pub fn new(
        subscriber_id: SubscriberId,
        session_id: SessionId,
        topic_id: String,
        queue_id: String,
    ) -> Self {
        Self {
            subscriber_id,
            start_delivery_time: DateTimeAsMicroseconds::now(),
            delivered_amount: MetricOneSecond::new(),
            delivery_microseconds: MetricOneSecond::new(),
            active: 0,
            delivery_history: MetricsHistory::new(),
            session_id,
            topic_id,
            queue_id,
            delivery_mode: DELIVERY_STATE_READY_TO_DELIVER,
        }
    }

    pub fn one_second_tick(&mut self) {
        if self.active > 0 {
            self.active -= 1;
        }

        let delivered_amount = self.delivered_amount.get_and_reset();
        let delivery_microseconds = self.delivery_microseconds.get_and_reset();

        if delivered_amount > 0 {
            let delivered = delivery_microseconds / delivered_amount;
            self.delivery_history.put(delivered as i32);
        }
    }

    pub fn set_delivered_statistic(
        &mut self,
        delivered_messages: usize,
        delivery_duration: Duration,
    ) {
        self.delivery_mode = DELIVERY_STATE_READY_TO_DELIVER;
        self.delivered_amount.increase(delivered_messages);
        self.delivery_microseconds
            .increase(delivery_duration.as_micros() as usize);
    }

    pub fn set_not_delivered_statistic(
        &mut self,

        delivered_messages: i32,
        delivery_duration: Duration,
    ) {
        self.delivery_mode = DELIVERY_STATE_READY_TO_DELIVER;

        let mut del_mess = 1;
        if delivered_messages != 0 {
            del_mess = delivered_messages;
        }

        let value = delivery_duration.as_micros() as i32 / -del_mess;
        self.delivery_history.put(value);
    }

    pub fn set_started_delivery(&mut self) {
        self.start_delivery_time = DateTimeAsMicroseconds::now();
        self.active = 2;
        self.delivery_mode = DELIVERY_STATE_ON_DELIVERY;
    }

    pub fn set_delivery_mode_as_ready_to_deliver(&mut self) {
        self.delivery_mode = DELIVERY_STATE_READY_TO_DELIVER;
    }

    pub fn set_delivery_mode_as_rented(&mut self) {
        self.delivery_mode = DELIVERY_STATE_RENTED;
    }

    pub fn set_delivery_mode_as_on_delivery(&mut self) {
        self.delivery_mode = DELIVERY_STATE_ON_DELIVERY;
    }
}
