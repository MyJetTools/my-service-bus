use std::collections::HashMap;

use serde::*;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MySbHttpContract {
    pub version: String,
    #[serde(rename = "persistenceVersion")]
    pub persistence_version: String,

    pub topics: TopicsHttpContract,
    pub queues: HashMap<String, TopicQueuesModel>,
    pub sessions: MySbSessionsHttpModel,
    pub system: SbSystemModel,
}

impl MySbHttpContract {
    pub fn get_session(&self, session_id: i64) -> Option<&MySbSessionHttpModel> {
        for session in self.sessions.items.iter() {
            if session_id == session.id {
                return Some(session);
            }
        }

        None
    }

    pub fn get_publishers_and_subscribers(
        &self,
        session_id: i64,
    ) -> (Vec<(String, i64)>, Vec<(String, String, i64)>) {
        let mut publishers: Vec<(String, i64)> = Vec::new();
        let mut subscribers: Vec<(String, String, i64)> = Vec::new();

        for topic in &self.topics.items {
            for publisher in &topic.publishers {
                if publisher.session_id == session_id {
                    publishers.push((topic.id.clone(), publisher.active));
                }
            }

            for subscriber in &topic.subscribers {
                if subscriber.session_id == session_id {
                    subscribers.push((
                        topic.id.clone(),
                        subscriber.queue_id.clone(),
                        subscriber.active,
                    ));
                }
            }
        }

        (publishers, subscribers)
    }
    pub fn get_status_bar_calculated_values(&self) -> StatusBarCalculatedValue {
        let mut result = StatusBarCalculatedValue {
            msg_per_sec: 0,
            persist_queue: 0,
            packets_per_sec: 0,
            total_pages_size: 0,
        };

        for topic in &self.topics.items {
            result.persist_queue += topic.persist_size;
            result.msg_per_sec += topic.messages_per_src;
            result.packets_per_sec += topic.packet_per_sec;

            for page in &topic.pages {
                result.total_pages_size += page.size;
            }
        }

        result
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TopicsHttpContract {
    pub items: Vec<TopicHttpModel>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TopicHttpModel {
    pub id: String,
    #[serde(rename = "messageId")]
    pub message_id: i64,
    #[serde(rename = "packetPerSec")]
    pub packet_per_sec: i64,
    #[serde(rename = "meanMessageSize")]
    pub mean_message_size: i64,
    #[serde(rename = "messagesPerSec")]
    pub messages_per_src: i64,

    #[serde(rename = "persistSize")]
    pub persist_size: i64,

    pub pages: Vec<SbMessagePageModel>,
    #[serde(rename = "publishHistory")]
    pub publish_history: Vec<i32>,

    pub publishers: Vec<TopicPublisher>,
    pub subscribers: Vec<TopicSubscriber>,

    #[serde(default)]
    pub deleted: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SbMessagePageModel {
    pub id: i64,
    pub amount: i64,
    pub size: i64,
    pub persist_size: i64,
    #[serde(rename = "subPages")]
    pub sub_pages: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TopicPublisher {
    #[serde(rename = "sessionId")]
    pub session_id: i64,
    pub active: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TopicSubscriber {
    pub id: i64,
    #[serde(rename = "sessionId")]
    pub session_id: i64,
    #[serde(rename = "queueId")]
    pub queue_id: String,
    pub active: i64,
    #[serde(rename = "deliveryState")]
    pub delivery_state: u8,
    #[serde(rename = "deliveryStateStr")]
    pub delivery_state_str: Option<String>,
    pub history: Vec<i32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TopicQueuesModel {
    pub queues: Vec<TopicQueueModel>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TopicQueueModel {
    pub id: String,
    #[serde(rename = "queueType")]
    pub queue_type: u8,
    pub size: i64,
    #[serde(rename = "onDelivery")]
    pub on_delivery: i64,
    pub data: Vec<QueueIntervalModel>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct QueueIntervalModel {
    #[serde(rename = "fromId")]
    pub from_id: i64,
    #[serde(rename = "toId")]
    pub to_id: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct SbSystemModel {
    pub usedmem: i64,
    pub totalmem: i64,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MySbSessionsHttpModel {
    pub items: Vec<MySbSessionHttpModel>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct MySbSessionHttpModel {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub tp: Option<String>,
    pub ip: String,
    pub version: Option<String>,
    #[serde(rename = "envInfo")]
    pub env_info: Option<String>,
    pub connected: String,
    #[serde(rename = "lastIncoming")]
    pub last_incoming: String,
    #[serde(rename = "readSize")]
    pub read_size: i64,
    #[serde(rename = "writtenSize")]
    pub written_size: i64,
    #[serde(rename = "readPerSec")]
    pub read_per_sec: i64,
    #[serde(rename = "writtenPerSec")]
    pub written_per_sec: i64,
}

impl MySbSessionHttpModel {
    pub fn get_session_as_string(&self) -> &str {
        match &self.version {
            Some(version) => version,
            None => "???",
        }
    }

    pub fn filter_me(&self, filter_string: &str) -> bool {
        if filter_string.is_empty() {
            return true;
        }

        if self.name.to_lowercase().contains(filter_string) {
            return true;
        }

        false
    }

    pub fn get_session_type(&self) -> SessionType {
        match &self.tp {
            Some(session_type) => {
                if session_type.starts_with("tcp") {
                    SessionType::Tcp
                } else {
                    SessionType::Http
                }
            }
            None => SessionType::Tcp,
        }
    }
}

pub enum SessionType {
    Tcp,
    Http,
}

pub struct StatusBarCalculatedValue {
    pub msg_per_sec: i64,
    pub packets_per_sec: i64,
    pub persist_queue: i64,
    pub total_pages_size: i64,
}
