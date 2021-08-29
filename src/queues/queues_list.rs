use std::{collections::HashMap, sync::Arc};

use my_service_bus_shared::queue_with_intervals::QueueWithIntervals;
use tokio::sync::RwLock;

use crate::topics::TopicQueueSnapshot;

use super::{
    queue::{TopicQueue, TopicQueueMonitoringData},
    TopicQueueType,
};

pub struct TopicQueueListData {
    queues: HashMap<String, Arc<TopicQueue>>,
    snapshot_id: usize,
}

pub struct TopicQueuesList {
    data: RwLock<TopicQueueListData>,
}

impl TopicQueuesList {
    pub fn new() -> Self {
        let data = TopicQueueListData {
            queues: HashMap::new(),
            snapshot_id: 0,
        };

        TopicQueuesList {
            data: RwLock::new(data),
        }
    }

    pub async fn add_queue_if_not_exists(
        &self,
        topic_id: &str,
        queue_id: &str,
        queue_type: TopicQueueType,
    ) -> Arc<TopicQueue> {
        let mut write_access = self.data.write().await;

        if !write_access.queues.contains_key(queue_id) {
            let queue = TopicQueue::new(topic_id, queue_id, queue_type);

            let queue = Arc::new(queue);
            write_access
                .queues
                .insert(queue_id.to_string(), queue.clone());

            write_access.snapshot_id += 1;
        }

        let result = write_access.queues.get(queue_id).unwrap();

        result.update_queue_type(queue_type).await;

        return result.clone();
    }

    pub async fn restore(
        &self,
        topic_id: &str,
        queue_id: &str,
        queue_type: TopicQueueType,
        queue: QueueWithIntervals,
    ) -> Arc<TopicQueue> {
        let topic_queue = TopicQueue::restore(topic_id, queue_id, queue_type, queue);
        let topic_queue = Arc::new(topic_queue);

        let mut write_access = self.data.write().await;

        write_access
            .queues
            .insert(queue_id.to_string(), topic_queue.clone());

        write_access.snapshot_id += 1;

        topic_queue
    }

    pub async fn get(&self, queue_id: &str) -> Option<Arc<TopicQueue>> {
        let read_access = self.data.read().await;

        match read_access.queues.get(queue_id) {
            Some(result) => Some(Arc::clone(result)),
            None => None,
        }
    }

    pub async fn delete_queue(&self, queue_id: &str) -> Option<Arc<TopicQueue>> {
        let mut write_access = self.data.write().await;
        let result = write_access.queues.remove(queue_id);
        write_access.snapshot_id += 1;
        result
    }

    pub async fn get_queues(&self) -> Vec<Arc<TopicQueue>> {
        let mut result = Vec::new();

        let read_access = self.data.read().await;

        for queue in read_access.queues.values() {
            result.push(Arc::clone(queue));
        }

        result
    }

    pub async fn get_snapshot(&self) -> Vec<TopicQueueSnapshot> {
        let mut result = Vec::new();

        let read_access = self.data.read().await;

        for queue in read_access.queues.values() {
            result.push(queue.get_snapshot().await);
        }
        return result;
    }

    pub async fn get_queues_with_snapshot_id(&self) -> (usize, Vec<Arc<TopicQueue>>) {
        let mut result = Vec::new();

        let read_access = self.data.read().await;

        for queue in read_access.queues.values() {
            result.push(Arc::clone(queue));
        }

        (read_access.snapshot_id, result)
    }

    pub async fn get_monitoring_data(&self) -> (usize, Vec<TopicQueueMonitoringData>) {
        let queues = self.get_queues_with_snapshot_id().await;

        let mut monitoring_data = Vec::new();

        for queue in queues.1 {
            let read_access = queue.data.read().await;

            let item = TopicQueueMonitoringData {
                id: queue.queue_id.to_string(),
                queue_type: read_access.queue_type,
                size: read_access.queue.len(),
                queue: read_access.get_snapshot(),
            };

            monitoring_data.push(item);
        }

        return (queues.0, monitoring_data);
    }
}
