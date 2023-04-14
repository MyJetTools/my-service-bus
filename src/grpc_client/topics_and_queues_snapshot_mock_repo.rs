use tokio::sync::Mutex;

use crate::topics::TopicSnapshot;

use super::PersistenceError;

pub struct TopicsAndQueuesSnapshotMockRepo {
    pub snapshot: Mutex<Vec<TopicSnapshot>>,
}

impl TopicsAndQueuesSnapshotMockRepo {
    pub fn new() -> Self {
        TopicsAndQueuesSnapshotMockRepo {
            snapshot: Mutex::new(Vec::new()),
        }
    }
    pub async fn load(&self) -> Result<Vec<TopicSnapshot>, PersistenceError> {
        let snapshot = self.snapshot.lock().await;
        Ok(snapshot.clone())
    }
    pub async fn save(&self, snapshot: Vec<TopicSnapshot>) -> Result<(), PersistenceError> {
        let mut snapshot_access = self.snapshot.lock().await;
        *snapshot_access = snapshot;
        Ok(())
    }
}
