use std::{collections::HashMap, sync::Arc};

use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::validators::InvalidTopicName;
use tokio::sync::RwLock;

use super::topic::Topic;

pub struct TopicListData {
    pub topics: HashMap<String, Arc<Topic>>,
    pub snapshot_id: usize,
}

impl TopicListData {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            snapshot_id: 0,
        }
    }
}

pub struct TopicsList {
    data: RwLock<TopicListData>,
}

impl TopicsList {
    pub fn new() -> Self {
        TopicsList {
            data: RwLock::new(TopicListData::new()),
        }
    }

    pub async fn get(&self, topic_id: &str) -> Option<Arc<Topic>> {
        let read_access = self.data.read().await;

        match read_access.topics.get(topic_id) {
            Some(result) => Some(result.clone()),
            None => None,
        }
    }

    pub async fn get_all(&self) -> Vec<Arc<Topic>> {
        let mut result = Vec::new();
        let read_access = self.data.read().await;

        for topic in read_access.topics.values() {
            result.push(Arc::clone(topic))
        }

        result
    }

    pub async fn get_all_with_snapshot_id(&self) -> (usize, Vec<Arc<Topic>>) {
        let mut result = Vec::new();
        let read_access = self.data.read().await;

        for topic in read_access.topics.values() {
            result.push(Arc::clone(topic))
        }

        (read_access.snapshot_id, result)
    }

    pub async fn add_if_not_exists(&self, topic_id: &str) -> Result<Arc<Topic>, InvalidTopicName> {
        let mut write_access = self.data.write().await;

        if !write_access.topics.contains_key(topic_id) {
            my_service_bus_shared::validators::validate_topic_name(topic_id)?;

            let topic = Topic::new(topic_id.to_string(), 0);
            write_access
                .topics
                .insert(topic_id.to_string(), Arc::new(topic));
            write_access.snapshot_id += 1;
        }

        let result = write_access.topics.get(topic_id).unwrap();

        return Ok(result.clone());
    }

    pub async fn restore(&self, topic_id: String, message_id: MessageId) -> Arc<Topic> {
        let mut write_access = self.data.write().await;

        let topic = Topic::new(topic_id.to_string(), message_id);
        let result = Arc::new(topic);

        write_access.topics.insert(topic_id, result.clone());

        write_access.snapshot_id += 1;

        return result;
    }

    pub async fn one_second_tick(&self) {
        let topics = self.get_all().await;

        for topic in topics {
            topic.one_second_tick().await;
        }
    }
}
