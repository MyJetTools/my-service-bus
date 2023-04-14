use crate::{settings::SettingsModel, topics::TopicSnapshot};

#[cfg(test)]
use super::topics_and_queues_snapshot_mock_repo::TopicsAndQueuesSnapshotMockRepo;

use super::{
    topics_and_queues_snapshot_grpc_repo::TopcsAndQueuesSnapshotGrpcRepo, PersistenceError,
};

pub enum TopicsAndQueuesSnapshotRepo {
    Grpc(TopcsAndQueuesSnapshotGrpcRepo),
    #[cfg(test)]
    Mock(TopicsAndQueuesSnapshotMockRepo),
}

impl TopicsAndQueuesSnapshotRepo {
    pub async fn create_production_instance(settings: &SettingsModel) -> Self {
        let grpc_repo =
            TopcsAndQueuesSnapshotGrpcRepo::new(settings.persistence_grpc_url.to_string()).await;
        Self::Grpc(grpc_repo)
    }

    #[cfg(test)]
    pub fn create_mock_instance() -> Self {
        Self::Mock(TopicsAndQueuesSnapshotMockRepo::new())
    }

    pub async fn load(&self) -> Result<Vec<TopicSnapshot>, PersistenceError> {
        match self {
            TopicsAndQueuesSnapshotRepo::Grpc(repo) => repo.load().await,
            #[cfg(test)]
            TopicsAndQueuesSnapshotRepo::Mock(repo) => repo.load().await,
        }
    }
    pub async fn save(&self, snapshot: Vec<TopicSnapshot>) -> Result<(), PersistenceError> {
        match self {
            TopicsAndQueuesSnapshotRepo::Grpc(repo) => repo.save(snapshot).await,
            #[cfg(test)]
            TopicsAndQueuesSnapshotRepo::Mock(repo) => repo.save(snapshot).await,
        }
    }
}
