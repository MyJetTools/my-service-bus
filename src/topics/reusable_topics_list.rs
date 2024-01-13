use std::sync::Arc;

use super::Topic;

pub struct ReusableTopicsList {
    data: Vec<Arc<Topic>>,
    snapshot_id: Option<usize>,
}

impl ReusableTopicsList {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            snapshot_id: None,
        }
    }

    pub fn check_with_snapshot_id(&self, value: usize) -> bool {
        match self.snapshot_id {
            Some(snapshot_id) => snapshot_id == value,
            None => false,
        }
    }

    pub fn clean(&mut self, capacity: usize) {
        self.data.clear();
        self.data.shrink_to(capacity);
    }

    pub fn push(&mut self, topic: Arc<Topic>) {
        self.data.push(topic);
    }

    pub fn update_snapshot_id(&mut self, snapshot_id: usize) {
        self.snapshot_id = Some(snapshot_id);
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn iter(&self) -> std::slice::Iter<Arc<Topic>> {
        self.data.iter()
    }
}
