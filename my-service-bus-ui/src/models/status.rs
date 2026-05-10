use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StatusJsonResult {
    pub topics: TopicsJsonResult,
    pub queues: BTreeMap<String, QueuesJsonResult>,
    pub sessions: SessionsJsonResult,
    pub system: SystemStatusModel,
    #[serde(rename = "persistenceVersion")]
    pub persistence_version: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SystemStatusModel {
    pub usedmem: u64,
    pub totalmem: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopicsJsonResult {
    pub items: Vec<TopicJsonContract>,
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TopicJsonContract {
    pub id: String,
    #[serde(rename = "messageId")]
    pub message_id: i64,
    #[serde(rename = "packetPerSec")]
    pub packets_per_second: usize,
    #[serde(rename = "messagesPerSec")]
    pub messages_per_second: usize,
    #[serde(rename = "meanMessageSize")]
    pub mean_message_size: usize,
    #[serde(rename = "persistSize")]
    pub persist_size: usize,
    pub persist: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueuesJsonResult {
    pub queues: Vec<QueueJsonContract>,
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct QueueJsonContract {
    pub id: String,
    #[serde(rename = "queueType")]
    pub queue_type: u8,
    pub size: usize,
    #[serde(rename = "onDelivery")]
    pub on_delivery: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionsJsonResult {
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
    pub items: Vec<SessionJsonResult>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SessionJsonResult {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub session_type: String,
    pub ip: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(rename = "envInfo", default)]
    pub env_info: Option<String>,
    pub connected: String,
    #[serde(rename = "lastIncoming")]
    pub last_incoming: String,
    #[serde(rename = "readPerSec")]
    pub read_per_sec: usize,
    #[serde(rename = "writtenPerSec")]
    pub written_per_sec: usize,
}
