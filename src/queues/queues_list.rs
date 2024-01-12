use my_service_bus::abstractions::{
    queue_with_intervals::QueueWithIntervals, subscriber::TopicQueueType,
};

use crate::{queue_subscribers::QueueSubscriber, sessions::SessionId, topics::TopicQueueSnapshot};

use super::queue::TopicQueue;

pub struct TopicQueuesList {
    queues: Vec<TopicQueue>,
    snapshot_id: usize,
}

impl TopicQueuesList {
    pub fn new() -> Self {
        TopicQueuesList {
            queues: Vec::new(),
            snapshot_id: 0,
        }
    }

    pub fn get_snapshot_id(&self) -> usize {
        self.snapshot_id
    }

    fn has_the_queue(&self, queue_id: &str) -> bool {
        for queue in &self.queues {
            if queue.queue_id == queue_id {
                return true;
            }
        }

        return false;
    }

    pub fn get(&self, queue_id: &str) -> Option<&TopicQueue> {
        for queue in &self.queues {
            if queue.queue_id == queue_id {
                return Some(queue);
            }
        }

        return None;
    }

    pub fn get_mut(&mut self, queue_id: &str) -> Option<&mut TopicQueue> {
        for queue in &mut self.queues {
            if queue.queue_id == queue_id {
                return Some(queue);
            }
        }

        return None;
    }

    pub fn add_queue_if_not_exists(
        &mut self,
        topic_id: String,
        queue_id: String,
        queue_type: TopicQueueType,
    ) -> &mut TopicQueue {
        if !self.has_the_queue(queue_id.as_str()) {
            let queue = TopicQueue::new(topic_id, queue_id.to_string(), queue_type);

            self.queues.push(queue);

            self.snapshot_id += 1;
        }

        let result = self.get_mut(queue_id.as_str()).unwrap();

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

        self.queues.push(topic_queue);

        self.snapshot_id += 1;

        return self.get(queue_id.as_str()).unwrap();
    }

    pub fn remove(&mut self, queue_id: &str) -> Option<TopicQueue> {
        let index = self.queues.iter().position(|x| x.queue_id == queue_id)?;
        let result = self.queues.remove(index);
        self.snapshot_id += 1;
        Some(result)
    }

    pub fn get_all(&self) -> impl Iterator<Item = &TopicQueue> {
        self.queues.iter()
    }

    pub fn get_all_mut(&mut self) -> impl Iterator<Item = &mut TopicQueue> {
        self.queues.iter_mut()
    }

    pub fn get_snapshot_to_persist(&self) -> Vec<TopicQueueSnapshot> {
        let mut result = Vec::new();

        for queue in self.get_all() {
            let get_snapshot_to_persist_result = queue.get_snapshot_to_persist();

            if let Some(snapshot_to_persist) = get_snapshot_to_persist_result {
                result.push(snapshot_to_persist);
            }
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
