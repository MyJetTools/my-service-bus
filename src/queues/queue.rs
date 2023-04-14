use my_service_bus_abstractions::{
    queue_with_intervals::{QueueIndexRange, QueueWithIntervals},
    subscriber::TopicQueueType,
    MessageId,
};
use rust_extensions::date_time::DateTimeAsMicroseconds;
use tokio::sync::Mutex;

use crate::{
    operations::OperationFailResult,
    queue_subscribers::{QueueSubscriber, SubscriberId, SubscribersList},
    topics::TopicQueueSnapshot,
};

use super::{delivery_attempts::DeliveryAttempts, DeliveryBucket};

pub struct TopicQueue {
    pub topic_id: String,
    pub queue_id: String,
    pub queue: QueueWithIntervals,
    pub subscribers: SubscribersList,
    pub delivery_attempts: DeliveryAttempts,
    pub queue_type: TopicQueueType,

    pub delivery_lock: Mutex<usize>,
}

impl TopicQueue {
    pub fn new(topic_id: String, queue_id: String, queue_type: TopicQueueType) -> Self {
        Self {
            topic_id,
            queue_id,
            queue: QueueWithIntervals::new(),
            subscribers: SubscribersList::new(queue_type),
            delivery_attempts: DeliveryAttempts::new(),
            queue_type,
            delivery_lock: Mutex::new(0),
        }
    }

    pub fn restore(
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
        queue: QueueWithIntervals,
    ) -> Self {
        Self {
            topic_id,
            queue_id,
            queue,
            subscribers: SubscribersList::new(queue_type),
            delivery_attempts: DeliveryAttempts::new(),
            queue_type,
            delivery_lock: Mutex::new(0),
        }
    }

    pub fn get_min_msg_id(&self) -> Option<MessageId> {
        MessageId::from_opt_i64(self.queue.get_min_id())
    }

    pub fn get_snapshot_to_persist(&self) -> Option<TopicQueueSnapshot> {
        match self.queue_type {
            TopicQueueType::Permanent => {
                let result = TopicQueueSnapshot {
                    queue_id: self.queue_id.to_string(),
                    queue_type: self.queue_type.clone(),
                    ranges: self.queue.get_snapshot(),
                };

                Some(result)
            }
            TopicQueueType::DeleteOnDisconnect => None,
            TopicQueueType::PermanentWithSingleConnection => {
                let result = TopicQueueSnapshot {
                    queue_id: self.queue_id.to_string(),
                    queue_type: self.queue_type.clone(),
                    ranges: self.queue.get_snapshot(),
                };

                Some(result)
            }
        }
    }

    pub fn enqueue_messages(&mut self, msgs: &QueueWithIntervals) {
        for msg_id in msgs {
            self.queue.enqueue(msg_id);
        }
    }

    pub fn update_queue_type(&mut self, queue_type: TopicQueueType) {
        if !self.queue_type_is_about_to_change(queue_type) {
            return;
        }
        self.queue_type = queue_type;
    }

    pub fn get_queue_size(&self) -> i64 {
        return self.queue.len();
    }

    pub fn get_on_delivery(&self) -> i64 {
        self.subscribers.get_on_delivery_amount()
    }

    pub fn one_second_tick(&mut self) {
        self.subscribers.one_second_tick();
    }

    pub fn set_message_id(&mut self, message_id: MessageId, max_message_id: MessageId) {
        let mut intervals = Vec::new();

        intervals.push(QueueIndexRange {
            from_id: message_id.into(),
            to_id: max_message_id.into(),
        });

        self.queue.reset(intervals);
    }

    pub fn mark_not_delivered(&mut self, delivery_bucket: &DeliveryBucket) {
        self.process_not_delivered(&delivery_bucket.ids);
    }

    fn process_not_delivered(&mut self, not_delivered_ids: &QueueWithIntervals) {
        self.queue.merge_with(not_delivered_ids);

        for msg_id in not_delivered_ids {
            self.delivery_attempts.add(msg_id.into());
        }
    }

    pub fn confirmed_delivered(
        &mut self,
        subscriber_id: SubscriberId,
    ) -> Result<(), OperationFailResult> {
        let subscriber = self.subscribers.get_by_id_mut(subscriber_id);

        if subscriber.is_none() {
            return Err(OperationFailResult::SubscriberNotFound { id: subscriber_id });
        }

        let subscriber = subscriber.unwrap();

        let messages_bucket = subscriber.reset_delivery();

        if messages_bucket.is_none() {
            println!(
                "{}/{} confirmed_delivered: No messages on delivery at subscriber {}",
                self.topic_id,
                self.queue_id,
                subscriber_id.get_value()
            );

            return Ok(());
        };

        let messages_bucket = messages_bucket.unwrap();

        if messages_bucket.ids.len() == 0 {
            println!(
                "{}/{} confirmed_delivered: No messages on delivery at subscriber {}",
                self.topic_id,
                self.queue_id,
                subscriber_id.get_value()
            );

            return Ok(());
        };

        update_delivery_time(subscriber, messages_bucket.confirmed, true);

        self.process_delivered(&messages_bucket.ids);

        Ok(())
    }

    fn process_delivered(&mut self, delivered_ids: &QueueWithIntervals) {
        for msg_id in delivered_ids {
            self.delivery_attempts.reset(msg_id.into());
        }
    }

    pub fn confirmed_non_delivered(
        &mut self,
        subscriber_id: SubscriberId,
    ) -> Result<(), OperationFailResult> {
        let subscriber = self.subscribers.get_by_id_mut(subscriber_id);

        if subscriber.is_none() {
            return Err(OperationFailResult::SubscriberNotFound { id: subscriber_id });
        }

        let subscriber = subscriber.unwrap();

        let messages_bucket = subscriber.reset_delivery();

        if messages_bucket.is_none() {
            println!(
                "{}/{} confirmed_non_delivered: No messages on delivery at subscriber {}",
                self.topic_id,
                self.queue_id,
                subscriber_id.get_value()
            );

            return Ok(());
        };

        let messages_bucket = messages_bucket.unwrap();

        update_delivery_time(subscriber, messages_bucket.confirmed, false);

        self.process_not_delivered(&messages_bucket.ids);

        Ok(())
    }

    pub fn confirmed_some_delivered(
        &mut self,
        subscriber_id: SubscriberId,
        delivered: QueueWithIntervals,
    ) -> Result<(), OperationFailResult> {
        let subscriber = self.subscribers.get_by_id_mut(subscriber_id);

        if subscriber.is_none() {
            return Err(OperationFailResult::SubscriberNotFound { id: subscriber_id });
        }

        let subscriber = subscriber.unwrap();

        let delivery_bucket = subscriber.reset_delivery();

        if delivery_bucket.is_none() {
            println!(
                "{}/{} confirmed_some_delivered: No messages on delivery at subscriber {}",
                self.topic_id,
                self.queue_id,
                subscriber_id.get_value()
            );

            return Ok(());
        };

        let mut delivery_bucket = delivery_bucket.unwrap();

        //Remove delivered and what remains - is not delivered
        delivery_bucket.confirmed(&delivered);

        update_delivery_time(subscriber, delivery_bucket.confirmed, false);

        if delivery_bucket.ids.len() > 0 {
            self.process_not_delivered(&delivery_bucket.ids);
        } else {
            self.process_delivered(&delivered);
        }

        Ok(())
    }

    pub fn intermediary_confirmed(
        &mut self,
        subscriber_id: SubscriberId,
        confirmed: QueueWithIntervals,
    ) -> Result<(), OperationFailResult> {
        let subscriber = self.subscribers.get_by_id_mut(subscriber_id);

        if subscriber.is_none() {
            return Err(OperationFailResult::SubscriberNotFound { id: subscriber_id });
        }

        let subscriber = subscriber.unwrap();

        if confirmed.len() > 0 {
            subscriber.intermediary_confirmed(&confirmed);
            self.process_delivered(&confirmed);
        }

        return Ok(());
    }

    pub fn get_messages_on_delivery(
        &self,
        subscriber_id: SubscriberId,
    ) -> Option<QueueWithIntervals> {
        let subscriber = self.subscribers.get_by_id(subscriber_id)?;
        return subscriber.get_messages_on_delivery();
    }

    pub fn queue_type_is_about_to_change(&self, new_queue_type: TopicQueueType) -> bool {
        match self.queue_type {
            TopicQueueType::Permanent => match new_queue_type {
                TopicQueueType::Permanent => false,
                TopicQueueType::DeleteOnDisconnect => true,
                TopicQueueType::PermanentWithSingleConnection => true,
            },
            TopicQueueType::DeleteOnDisconnect => match new_queue_type {
                TopicQueueType::Permanent => true,
                TopicQueueType::DeleteOnDisconnect => false,
                TopicQueueType::PermanentWithSingleConnection => true,
            },
            TopicQueueType::PermanentWithSingleConnection => match new_queue_type {
                TopicQueueType::Permanent => true,
                TopicQueueType::DeleteOnDisconnect => true,
                TopicQueueType::PermanentWithSingleConnection => false,
            },
        }
    }

    pub fn is_permanent(&self) -> bool {
        match &self.queue_type {
            TopicQueueType::Permanent => true,
            TopicQueueType::DeleteOnDisconnect => false,
            TopicQueueType::PermanentWithSingleConnection => true,
        }
    }
}

fn update_delivery_time(subscriber: &mut QueueSubscriber, amount: usize, positive: bool) {
    let delivery_duration = DateTimeAsMicroseconds::now()
        .duration_since(subscriber.metrics.start_delivery_time)
        .as_positive_or_zero();

    if positive {
        subscriber
            .metrics
            .set_delivered_statistic(amount, delivery_duration);
    } else {
        subscriber
            .metrics
            .set_not_delivered_statistic(amount as i32, delivery_duration);
    }
}
