use std::sync::Arc;

use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::validators::InvalidTopicName;
use tokio::sync::RwLock;

use super::{topic::Topic, ReusableTopicsList, TopicListInner};

pub struct TopicsList {
    data: RwLock<TopicListInner>,
}

impl TopicsList {
    pub fn new() -> Self {
        TopicsList {
            data: RwLock::new(TopicListInner::new()),
        }
    }

    pub async fn get(&self, topic_id: &str) -> Option<Arc<Topic>> {
        let read_access = self.data.read().await;

        read_access.get(topic_id)
    }

    pub async fn get_all(&self) -> Vec<Arc<Topic>> {
        let read_access = self.data.read().await;
        read_access.get_all()
    }

    pub async fn fill_topics(&self, dest: &mut ReusableTopicsList) {
        let read_access = self.data.read().await;

        if dest.get_snapshot_id() == read_access.get_snapshot_id()
            && dest.len() == read_access.len()
        {
            return;
        }

        read_access.fill_with_topics(dest);
    }

    pub async fn get_all_with_snapshot_id(&self) -> (usize, Vec<Arc<Topic>>) {
        let read_access = self.data.read().await;
        (read_access.get_snapshot_id(), read_access.get_all())
    }

    pub async fn add_if_not_exists(&self, topic_id: &str) -> Result<Arc<Topic>, InvalidTopicName> {
        let mut write_access = self.data.write().await;
        write_access.add_if_not_exists(topic_id, true)
    }

    pub async fn restore(
        &self,
        topic_id: &str,
        message_id: MessageId,
        persist: bool,
    ) -> Arc<Topic> {
        let mut write_access = self.data.write().await;
        write_access.restore(topic_id, message_id, persist)
    }

    pub async fn delete_topic(&self, topic_id: &str) -> Option<Arc<Topic>> {
        let mut write_access = self.data.write().await;
        write_access.delete_topic(topic_id)
    }
}
