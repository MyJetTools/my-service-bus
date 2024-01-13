use std::sync::atomic::AtomicUsize;

use super::ReusableTopicsList;

pub struct AtomicSnapshotId {
    snapshot_id: AtomicUsize,
    len: AtomicUsize,
}

impl AtomicSnapshotId {
    pub fn new() -> Self {
        Self {
            snapshot_id: AtomicUsize::new(0),
            len: AtomicUsize::new(0),
        }
    }

    pub fn get_snapshot_id(&self) -> usize {
        self.snapshot_id.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn get_len(&self) -> usize {
        self.len.load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn update_snapshot_id(&self, value: usize) {
        self.snapshot_id
            .store(value, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn update_len(&self, value: usize) {
        self.len.store(value, std::sync::atomic::Ordering::Relaxed);
    }

    pub fn has_the_same_snapshot_id(&self, reusable_topic_id: &ReusableTopicsList) -> bool {
        self.get_snapshot_id() == reusable_topic_id.get_snapshot_id()
            && self.get_len() == reusable_topic_id.len()
    }
}
