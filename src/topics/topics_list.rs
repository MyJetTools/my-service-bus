use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;
use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::validators::InvalidTopicName;
use rust_extensions::sorted_vec::SortedVecOfArcWithStrKey;

use super::topic::Topic;

#[derive(Clone)]
pub struct TopicsSnapshot {
    pub topics: Arc<Vec<Arc<Topic>>>,
    pub snapshot_id: usize,
}

struct TopicsInner {
    sorted: SortedVecOfArcWithStrKey<Topic>,
    as_vec: Arc<Vec<Arc<Topic>>>,
    snapshot_id: usize,
}

impl TopicsInner {
    fn empty() -> Self {
        Self {
            sorted: SortedVecOfArcWithStrKey::new(),
            as_vec: Arc::new(Vec::new()),
            snapshot_id: 0,
        }
    }

    fn new(sorted: SortedVecOfArcWithStrKey<Topic>, snapshot_id: usize) -> Self {
        let as_vec = Arc::new(sorted.as_slice().to_vec());
        Self {
            sorted,
            as_vec,
            snapshot_id,
        }
    }
}

pub struct TopicsList {
    inner: ArcSwap<TopicsInner>,
    write_lock: Mutex<()>,
}

impl TopicsList {
    pub fn new() -> Self {
        Self {
            inner: ArcSwap::from_pointee(TopicsInner::empty()),
            write_lock: Mutex::new(()),
        }
    }

    pub fn get(&self, topic_id: &str) -> Option<Arc<Topic>> {
        self.inner.load().sorted.get(topic_id).cloned()
    }

    pub fn get_all(&self) -> Arc<Vec<Arc<Topic>>> {
        self.inner.load().as_vec.clone()
    }

    pub fn get_all_with_snapshot_id(&self) -> TopicsSnapshot {
        let guard = self.inner.load();
        TopicsSnapshot {
            topics: guard.as_vec.clone(),
            snapshot_id: guard.snapshot_id,
        }
    }

    pub fn add_if_not_exists(&self, topic_id: &str) -> Result<Arc<Topic>, InvalidTopicName> {
        my_service_bus::shared::validators::validate_topic_name(topic_id)?;

        let _guard = self.write_lock.lock().unwrap();
        let current = self.inner.load_full();

        if let Some(existing) = current.sorted.get(topic_id) {
            return Ok(existing.clone());
        }

        let topic = Arc::new(Topic::new(topic_id.to_string(), 0, true));
        let mut new_sorted = current.sorted.clone();
        new_sorted.insert_or_replace(topic.clone());

        self.inner.store(Arc::new(TopicsInner::new(
            new_sorted,
            current.snapshot_id + 1,
        )));

        Ok(topic)
    }

    pub fn add(&self, topic_id: &str, message_id: MessageId, persist: bool) -> Arc<Topic> {
        let _guard = self.write_lock.lock().unwrap();
        let current = self.inner.load_full();

        let topic = Arc::new(Topic::new(
            topic_id.to_string(),
            message_id.get_value(),
            persist,
        ));
        let mut new_sorted = current.sorted.clone();
        new_sorted.insert_or_replace(topic.clone());

        self.inner.store(Arc::new(TopicsInner::new(
            new_sorted,
            current.snapshot_id + 1,
        )));

        topic
    }

    pub fn delete_topic(&self, topic_id: &str) -> Option<Arc<Topic>> {
        let _guard = self.write_lock.lock().unwrap();
        let current = self.inner.load_full();

        let mut new_sorted = current.sorted.clone();
        let removed = new_sorted.remove(topic_id)?;

        self.inner.store(Arc::new(TopicsInner::new(
            new_sorted,
            current.snapshot_id + 1,
        )));

        Some(removed)
    }
}
