use std::sync::Arc;

use my_service_bus::abstractions::MessageId;
use rust_extensions::sorted_vec::SortedVecOfArcWithStrKey;

use super::Topic;

pub struct TopicListInner {
    topics: SortedVecOfArcWithStrKey<Topic>,
}

impl TopicListInner {
    pub fn new() -> Self {
        Self {
            topics: SortedVecOfArcWithStrKey::new(),
        }
    }

    pub fn get(&self, topic_id: &str) -> Option<Arc<Topic>> {
        self.topics.get(topic_id).cloned()
    }

    pub fn add_if_not_exists(&mut self, topic_id: &str, persist: bool) -> AddIfNotExistsResult {
        let new_one = match self.topics.get_or_create(topic_id) {
            rust_extensions::sorted_vec::GetOrCreateEntry::Get(topic) => {
                return AddIfNotExistsResult::NotAdded(topic.clone());
            }
            rust_extensions::sorted_vec::GetOrCreateEntry::Create(entry) => {
                let topic = Topic::new(topic_id.to_string(), 0, persist);
                let topic = Arc::new(topic);
                let result = entry.insert_and_get_value(topic);
                result.clone()
            }
        };

        AddIfNotExistsResult::Added {
            snapshot: self.topics.as_slice().to_vec(),
            added_topic: new_one,
        }
    }

    pub fn add(&mut self, topic_id: &str, message_id: MessageId, persist: bool) -> AddTopicResult {
        let topic = Topic::new(topic_id.to_string(), message_id.get_value(), persist);
        let topic = Arc::new(topic);
        self.topics.insert_or_replace(topic.clone());

        AddTopicResult {
            topic,
            snapshot: self.topics.as_slice().to_vec(),
        }
    }

    pub fn remove(&mut self, topic_id: &str) -> Option<RemoveTopicResult> {
        let removed_topic = self.topics.remove(topic_id)?;
        let result = RemoveTopicResult {
            removed_topic,
            snapshot: self.topics.as_slice().to_vec(),
        };

        Some(result)
    }
}

pub enum AddIfNotExistsResult {
    Added {
        snapshot: Vec<Arc<Topic>>,
        added_topic: Arc<Topic>,
    },
    NotAdded(Arc<Topic>),
}

pub struct AddTopicResult {
    pub topic: Arc<Topic>,
    pub snapshot: Vec<Arc<Topic>>,
}

pub struct RemoveTopicResult {
    pub removed_topic: Arc<Topic>,
    pub snapshot: Vec<Arc<Topic>>,
}
