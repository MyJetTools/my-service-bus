use std::sync::Arc;

use super::Topic;

pub struct ReusableTopicsList {
    data: Vec<Arc<Topic>>,
    snapshot_id: usize,
}

impl ReusableTopicsList {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            snapshot_id: 0,
        }
    }

    pub fn get_snapshot_id(&self) -> usize {
        self.snapshot_id
    }

    pub fn clean(&mut self, capacity: usize) {
        self.data.clear();
        self.data.shrink_to(capacity);
    }

    pub fn push(&mut self, topic: Arc<Topic>) {
        self.data.push(topic);
    }

    pub fn update_snapshot_id(&mut self, snapshot_id: usize) {
        self.snapshot_id = snapshot_id;
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Arc<Topic>> {
        self.data.iter()
    }
}
