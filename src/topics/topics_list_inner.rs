use std::sync::Arc;

use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::validators::InvalidTopicName;
use rust_extensions::sorted_vec::SortedVecOfArcWithStrKey;

use super::{ReusableTopicsList, Topic};

pub struct TopicListInner {
    topics: SortedVecOfArcWithStrKey<Topic>,
    snapshot_id: usize,
}

impl TopicListInner {
    pub fn new() -> Self {
        Self {
            topics: SortedVecOfArcWithStrKey::new(),
            snapshot_id: 0,
        }
    }

    pub fn get(&self, topic_id: &str) -> Option<Arc<Topic>> {
        self.topics.get(topic_id).cloned()
    }

    pub fn fill_with_topics(&self, dest: &mut ReusableTopicsList) {
        dest.clean(self.topics.len());
        for topic in self.topics.iter() {
            dest.push(topic.clone());
        }

        dest.update_snapshot_id(self.snapshot_id);
    }
    pub fn get_all(&self) -> Vec<Arc<Topic>> {
        let mut result = Vec::with_capacity(self.topics.len());
        for topic in self.topics.iter() {
            result.push(topic.clone())
        }

        result
    }

    pub fn get_snapshot_id(&self) -> usize {
        self.snapshot_id
    }

    pub fn add_if_not_exists(
        &mut self,
        topic_id: &str,
        persist: bool,
    ) -> Result<Arc<Topic>, InvalidTopicName> {
        match self.topics.get_or_create(topic_id) {
            rust_extensions::sorted_vec::GetOrCreateEntry::Get(item) => Ok(item.clone()),
            rust_extensions::sorted_vec::GetOrCreateEntry::Create(entry) => {
                my_service_bus::shared::validators::validate_topic_name(topic_id)?;

                let topic = Topic::new(topic_id.to_string(), 0, persist);
                let topic = Arc::new(topic);
                let result = entry.insert_and_get_value(topic);
                self.snapshot_id += 1;
                return Ok(result.clone());
            }
        }
    }

    pub fn restore(&mut self, topic_id: &str, message_id: MessageId, persist: bool) -> Arc<Topic> {
        let topic = Topic::new(topic_id.to_string(), message_id.get_value(), persist);
        let result = Arc::new(topic);
        self.topics.insert_or_replace(result.clone());

        self.snapshot_id += 1;
        return result;
    }

    pub fn delete_topic(&mut self, topic_id: &str) -> Option<Arc<Topic>> {
        let result = self.topics.remove(topic_id);
        self.snapshot_id += 1;
        result
    }

    pub fn len(&self) -> usize {
        self.topics.len()
    }
}
