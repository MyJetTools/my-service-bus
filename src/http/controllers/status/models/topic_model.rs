use crate::{messages_page::MessagesPageList, topics::TopicData};

use my_http_server_swagger::MyHttpObjectStructure;
use serde::{Deserialize, Serialize};

use super::{
    topic_publisher::TopicPublisherJsonModel, topic_queue_subscriber::TopicQueueSubscriberJsonModel,
};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct TopicsJsonResult {
    pub items: Vec<TopicJsonContract>,
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct TopicJsonContract {
    pub id: String,
    #[serde(rename = "messageId")]
    pub message_id: i64,
    #[serde(rename = "packetPerSec")]
    pub packets_per_second: usize,
    #[serde(rename = "messagesPerSec")]
    pub messages_per_second: usize,
    pub pages: Vec<TopicPageJsonContract>,
    #[serde(rename = "persistSize")]
    pub persist_size: usize,
    #[serde(rename = "publishHistory")]
    pub publish_history: Vec<i32>,
    pub publishers: Vec<TopicPublisherJsonModel>,
    pub subscribers: Vec<TopicQueueSubscriberJsonModel>,
}

impl TopicJsonContract {
    pub fn new(topic_data: &TopicData) -> Self {
        let mut publishers = Vec::with_capacity(topic_data.publishers.len());

        let mut subscribers = Vec::new();

        for (session_id, active) in &topic_data.publishers {
            publishers.push(TopicPublisherJsonModel {
                session_id: *session_id,
                active: *active,
            });
        }

        for queue in topic_data.queues.get_all() {
            if let Some(queue_subscribers) = queue.subscribers.get_all() {
                for subscriber in queue_subscribers {
                    subscribers.push(TopicQueueSubscriberJsonModel::new(subscriber));
                }
            }
        }

        Self {
            id: topic_data.topic_id.to_string(),
            message_id: topic_data.message_id.into(),
            packets_per_second: topic_data.statistics.packets_per_second,
            messages_per_second: topic_data.statistics.messages_per_second,
            publish_history: topic_data.statistics.publish_history.get(),
            persist_size: topic_data.statistics.size_metrics.persist_size,
            publishers,
            pages: TopicPageJsonContract::as_vec(&topic_data.pages),
            subscribers,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct TopicPageJsonContract {
    pub id: i64,
    pub amount: usize,
    pub size: usize,
    pub persist_size: usize,
    #[serde(rename = "subPages")]
    pub sub_pages: Vec<i64>,
}

impl TopicPageJsonContract {
    pub fn as_vec(pages: &MessagesPageList) -> Vec<Self> {
        let mut result = Vec::new();

        for (page_id, page_metrics) in pages.get_page_size_metrics() {
            let size_metrics = page_metrics.get_size_metrics();

            result.push(TopicPageJsonContract {
                id: page_id,
                amount: size_metrics.messages_amount,
                size: size_metrics.data_size,
                persist_size: size_metrics.persist_size,
                sub_pages: page_metrics.get_sub_pages(),
            });
        }

        result
    }
}
