use std::collections::{
    hash_map::{Values, ValuesMut},
    HashMap,
};

use my_service_bus::abstractions::{
    queue_with_intervals::QueueWithIntervals, subscriber::TopicQueueType,
};

use crate::{queue_subscribers::QueueSubscriber, sessions::SessionId, topics::TopicQueueSnapshot};

use super::queue::TopicQueue;

pub struct TopicQueuesList {
    queues: HashMap<String, TopicQueue>,
    snapshot_id: usize,
}

impl TopicQueuesList {
    pub fn new() -> Self {
        TopicQueuesList {
            queues: HashMap::new(),
            snapshot_id: 0,
        }
    }

    pub fn get_snapshot_id(&self) -> usize {
        self.snapshot_id
    }

    pub fn add_queue_if_not_exists(
        &mut self,
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
    ) -> &mut TopicQueue {
        if !self.queues.contains_key(queue_id.as_str()) {
            let queue = TopicQueue::new(topic_id, queue_id.to_string(), queue_type);

            self.queues.insert(queue_id.to_string(), queue);

            self.snapshot_id += 1;
        }

        let result = self.queues.get_mut(queue_id.as_str()).unwrap();

        result.update_queue_type(queue_type);

        return result;
    }

    pub fn restore(
        &mut self,
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
        queue: QueueWithIntervals,
    ) -> &TopicQueue {
        let topic_queue = TopicQueue::restore(topic_id, queue_id.to_string(), queue_type, queue);

        self.queues.insert(queue_id.to_string(), topic_queue);

        self.snapshot_id += 1;

        return self.queues.get(queue_id.as_str()).unwrap();
    }

    pub fn get(&self, queue_id: &str) -> Option<&TopicQueue> {
        return self.queues.get(queue_id);
    }

    pub fn get_mut(&mut self, queue_id: &str) -> Option<&mut TopicQueue> {
        return self.queues.get_mut(queue_id);
    }

    pub fn delete_queue(&mut self, queue_id: &str) -> Option<TopicQueue> {
        let result = self.queues.remove(queue_id);
        self.snapshot_id += 1;
        result
    }

    pub fn get_queues(&self) -> Values<String, TopicQueue> {
        self.queues.values()
    }

    pub fn get_all(&self) -> Values<String, TopicQueue> {
        self.queues.values()
    }

    pub fn get_all_mut(&mut self) -> ValuesMut<String, TopicQueue> {
        self.queues.values_mut()
    }

    pub fn get_snapshot_to_persist(&self) -> Vec<TopicQueueSnapshot> {
        let mut result = Vec::new();

        for queue in self.queues.values() {
            let get_snapshot_to_persist_result = queue.get_snapshot_to_persist();

            if let Some(snapshot_to_persist) = get_snapshot_to_persist_result {
                result.push(snapshot_to_persist);
            }
        }
        return result;
    }

    pub fn remove(&mut self, queue_id: &str) -> Option<TopicQueue> {
        self.delete_queue(queue_id)
    }

    pub fn get_queues_with_no_subscribers(&self) -> Option<Vec<&TopicQueue>> {
        let mut result = None;

        for queue in self.queues.values() {
            if queue.subscribers.get_amount() > 0 {
                continue;
            }

            if result.is_none() {
                result = Some(Vec::new());
            }

            result.as_mut().unwrap().push(queue);
        }

        result
    }

    pub fn remove_subscribers_by_session_id(
        &mut self,
        session_id: SessionId,
    ) -> Option<Vec<(&mut TopicQueue, QueueSubscriber)>> {
        let mut result = None;

        for queue in self.queues.values_mut() {
            let remove_result = queue.subscribers.remove_by_session_id(session_id);
            if let Some(sub) = remove_result {
                if result.is_none() {
                    result = Some(Vec::new());
                }

                result.as_mut().unwrap().push((queue, sub));
            }
        }

        result
    }
}
