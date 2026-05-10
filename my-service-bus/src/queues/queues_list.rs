use my_service_bus::abstractions::{
    queue_with_intervals::QueueWithIntervals, subscriber::TopicQueueType,
};
use rust_extensions::sorted_vec::SortedVecWithStrKey;

use crate::{queue_subscribers::QueueSubscriber, sessions::SessionId, topics::TopicId};

use super::{queue::TopicQueue, QueueId};

pub struct TopicQueuesList {
    queues: SortedVecWithStrKey<TopicQueue>,
    snapshot_id: usize,
}

impl TopicQueuesList {
    pub fn new() -> Self {
        TopicQueuesList {
            queues: SortedVecWithStrKey::new(),
            snapshot_id: 0,
        }
    }

    pub fn get_snapshot_id(&self) -> usize {
        self.snapshot_id
    }

    pub fn get(&self, queue_id: &str) -> Option<&TopicQueue> {
        self.queues.get(queue_id)
    }

    pub fn get_mut(&mut self, queue_id: &str) -> Option<&mut TopicQueue> {
        self.queues.get_mut(queue_id)
    }

    pub fn add_queue_if_not_exists(
        &mut self,
        topic_id: TopicId,
        queue_id: String,
        queue_type: TopicQueueType,
    ) -> &mut TopicQueue {
        let index = match self.queues.insert_or_update(queue_id.as_str()) {
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Insert(entry) => {
                entry.insert_and_get_index(TopicQueue::new(topic_id, queue_id.into(), queue_type))
            }
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Update(entry) => {
                entry.item.update_queue_type(queue_type);
                entry.index
            }
        };

        self.snapshot_id += 1;

        return self.queues.get_by_index_mut(index).unwrap();
    }

    pub fn restore(
        &mut self,
        topic_id: TopicId,
        queue_id: QueueId,
        queue_type: TopicQueueType,
        queue: QueueWithIntervals,
    ) -> &TopicQueue {
        let topic_queue = TopicQueue::restore(topic_id, queue_id, queue_type, queue);

        let (index, _) = self.queues.insert_or_replace(topic_queue);

        self.snapshot_id += 1;

        self.queues.get_by_index(index).unwrap()
    }

    pub fn remove(&mut self, queue_id: &str) -> Option<TopicQueue> {
        let removed = self.queues.remove(queue_id);
        self.snapshot_id += 1;
        removed
    }

    pub fn get_all(&self) -> impl Iterator<Item = &TopicQueue> {
        self.queues.iter()
    }

    pub fn get_all_mut(&mut self) -> impl Iterator<Item = &mut TopicQueue> {
        self.queues.iter_mut()
    }

    pub fn get_snapshot<TResult>(&self, convert: impl Fn(&TopicQueue) -> TResult) -> Vec<TResult> {
        let mut result = Vec::with_capacity(self.queues.len());

        for item in self.get_all() {
            let item = convert(item);
            result.push(item);
        }
        return result;
    }

    pub fn get_queues_with_no_subscribers(&self) -> Option<Vec<&TopicQueue>> {
        let mut result = None;

        for queue in self.get_all() {
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

        for queue in self.get_all_mut() {
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
