use std::sync::Arc;

use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::validators::InvalidTopicName;
use tokio::sync::*;

use super::{topic::Topic, TopicListInner};

#[derive(Default, Clone)]
pub struct TopicsSnapshot {
    pub topics: Arc<Vec<Arc<Topic>>>,
    pub snapshot_id: usize,
}

impl TopicsSnapshot {
    pub fn update(&mut self, snapshot: Vec<Arc<Topic>>) {
        self.topics = Arc::new(snapshot);
        self.snapshot_id += 1;
    }
}

pub struct TopicsList {
    data: RwLock<TopicListInner>,
    snapshot: Mutex<TopicsSnapshot>,
}

impl TopicsList {
    pub fn new() -> Self {
        TopicsList {
            data: RwLock::new(TopicListInner::new()),
            snapshot: Default::default(),
        }
    }

    pub async fn get(&self, topic_id: &str) -> Option<Arc<Topic>> {
        let read_access = self.data.read().await;
        read_access.get(topic_id)
    }

    pub async fn get_all(&self) -> Arc<Vec<Arc<Topic>>> {
        let read_access = self.snapshot.lock().await;
        read_access.topics.clone()
    }

    pub async fn get_all_with_snapshot_id(&self) -> TopicsSnapshot {
        let read_access = self.snapshot.lock().await;
        read_access.clone()
    }

    pub async fn add_if_not_exists(&self, topic_id: &str) -> Result<Arc<Topic>, InvalidTopicName> {
        my_service_bus::shared::validators::validate_topic_name(topic_id)?;

        let add_result = {
            let mut write_access = self.data.write().await;
            write_access.add_if_not_exists(topic_id, true)
        };

        match add_result {
            super::AddIfNotExistsResult::Added {
                snapshot,
                added_topic,
            } => {
                self.snapshot.lock().await.update(snapshot);
                Ok(added_topic)
            }
            super::AddIfNotExistsResult::NotAdded(topic) => Ok(topic),
        }
    }

    pub async fn add(&self, topic_id: &str, message_id: MessageId, persist: bool) -> Arc<Topic> {
        let add_result = {
            let mut write_access = self.data.write().await;
            write_access.add(topic_id, message_id, persist)
        };

        self.snapshot.lock().await.update(add_result.snapshot);

        add_result.topic
    }

    pub async fn delete_topic(&self, topic_id: &str) -> Option<Arc<Topic>> {
        let remove_topic_result = {
            let mut write_access = self.data.write().await;
            write_access.remove(topic_id)
        }?;

        self.snapshot
            .lock()
            .await
            .update(remove_topic_result.snapshot);

        Some(remove_topic_result.removed_topic)
    }
}
