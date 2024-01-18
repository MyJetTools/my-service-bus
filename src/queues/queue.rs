use my_service_bus::abstractions::{
    queue_with_intervals::{QueueIndexRange, QueueWithIntervals},
    subscriber::TopicQueueType,
    MessageId,
};
use rust_extensions::sorted_vec::EntityWithStrKey;
use tokio::sync::Mutex;

use crate::{
    queue_subscribers::{SubscriberId, SubscribersList},
    topics::{TopicId, TopicQueueSnapshot},
};

use super::{delivery_attempts::DeliveryAttempts, QueueId};

pub struct TopicQueue {
    pub topic_id: TopicId,
    pub queue_id: QueueId,
    pub queue: QueueWithIntervals,
    pub subscribers: SubscribersList,
    pub delivery_attempts: DeliveryAttempts,
    pub queue_type: TopicQueueType,

    pub delivery_lock: Mutex<usize>,
}

impl EntityWithStrKey for TopicQueue {
    fn get_key(&self) -> &str {
        self.queue_id.as_str()
    }
}

impl TopicQueue {
    pub fn new(topic_id: TopicId, queue_id: QueueId, queue_type: TopicQueueType) -> Self {
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
        topic_id: TopicId,
        queue_id: QueueId,
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

    pub fn get_queue_size(&self) -> usize {
        return self.queue.queue_size();
    }

    pub fn get_on_delivery(&self) -> usize {
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

    pub fn confirm_delivered(&mut self, delivered_ids: &QueueWithIntervals) {
        for msg_id in delivered_ids {
            self.delivery_attempts.reset(msg_id.into());
        }
    }

    pub fn confirm_non_delivered(&mut self, ids: &QueueWithIntervals) {
        self.queue.merge_with(ids);

        for msg_id in ids {
            self.delivery_attempts.add(msg_id.into());
        }
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
