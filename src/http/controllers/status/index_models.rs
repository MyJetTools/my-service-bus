use std::collections::HashMap;

use crate::app::AppContext;

use serde::{Deserialize, Serialize};
use sysinfo::SystemExt;

use super::models::{
    queue_model::QueuesJsonResult,
    session_model::SessionsJsonResult,
    topic_model::{TopicJsonContract, TopicsJsonResult},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemStatusModel {
    usedmem: u64,
    totalmem: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusJsonResult {
    pub topics: TopicsJsonResult,
    pub queues: HashMap<String, QueuesJsonResult>,
    pub sessions: SessionsJsonResult,
    pub system: SystemStatusModel,
    #[serde(rename = "persistenceVersion")]
    pub persistence_version: String,
}

impl StatusJsonResult {
    pub async fn new(app: &AppContext) -> Self {
        let mut sys_info = sysinfo::System::new_all();

        sys_info.refresh_all();

        let (snapshot_id, all_topics) = app.topic_list.get_all_with_snapshot_id().await;

        let mut queues = HashMap::new();

        let mut topics = TopicsJsonResult {
            snapshot_id,
            items: Vec::new(),
        };

        let sessions = SessionsJsonResult::new(app).await;

        for topic in all_topics {
            let topic_data = topic.get_access().await;
            queues.insert(
                topic_data.topic_id.to_string(),
                QueuesJsonResult::new(&topic_data),
            );

            topics.items.push(TopicJsonContract::new(&topic_data));
        }

        let persistence_version = {
            let read_access = app.persistence_version.lock().await;
            read_access.to_string()
        };

        Self {
            topics,
            queues,
            sessions,
            system: SystemStatusModel {
                totalmem: sys_info.total_memory(),
                usedmem: sys_info.used_memory(),
            },
            persistence_version,
        }
    }
}
