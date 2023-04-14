use std::{collections::HashMap, sync::Arc};

use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::validators::InvalidTopicName;

use super::Topic;

pub struct TopicListInner {
    topics: HashMap<String, Arc<Topic>>,
    snapshot_id: usize,
}

impl TopicListInner {
    pub fn new() -> Self {
        Self {
            topics: HashMap::new(),
            snapshot_id: 0,
        }
    }

    pub fn get(&self, topic_id: &str) -> Option<Arc<Topic>> {
        self.topics.get(topic_id).cloned()
    }

    pub fn get_all(&self) -> Vec<Arc<Topic>> {
        let mut result = Vec::with_capacity(self.topics.len());
        for topic in self.topics.values() {
            result.push(topic.clone())
        }

        result
    }

    pub fn get_snapshot_id(&self) -> usize {
        self.snapshot_id
    }

    pub fn add_if_not_exists(&mut self, topic_id: &str) -> Result<Arc<Topic>, InvalidTopicName> {
        if !self.topics.contains_key(topic_id) {
            my_service_bus_shared::validators::validate_topic_name(topic_id)?;

            let topic = Topic::new(topic_id.to_string(), 0);
            let topic = Arc::new(topic);
            self.topics.insert(topic_id.to_string(), topic.clone());
            self.snapshot_id += 1;
            return Ok(topic);
        }

        let result = self.topics.get(topic_id).unwrap().clone();
        return Ok(result);
    }

    pub fn restore(&mut self, topic_id: String, message_id: MessageId) -> Arc<Topic> {
        let topic = Topic::new(topic_id.to_string(), message_id.get_value());
        let result = Arc::new(topic);
        self.topics.insert(topic_id, result.clone());

        self.snapshot_id += 1;
        return result;
    }

    pub fn delete_topic(&mut self, topic_id: &str) -> Option<Arc<Topic>> {
        let result = self.topics.remove(topic_id);
        self.snapshot_id += 1;
        result
    }
}
