use crate::topics::TopicData;

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
        let mut publishers = Vec::new();

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
                    subscribers.push(TopicQueueSubscriberJsonModel {
                        session_id: subscriber.session.id,
                        subscriber_id: subscriber.id,
                        delivery_state: subscriber.delivery_state.to_u8(),
                        history: subscriber.metrics.delivery_history.get(),
                        active: subscriber.metrics.active,
                        queue_id: queue.queue_id.to_string(),
                    });
                }
            }
        }

        Self {
            id: topic_data.topic_id.to_string(),
            message_id: topic_data.message_id.into(),
            packets_per_second: topic_data.metrics.packets_per_second,
            messages_per_second: topic_data.metrics.messages_per_second,
            publish_history: topic_data.metrics.publish_history.get(),
            persist_size: topic_data.metrics.size_metrics.persist_size,
            publishers,
            pages: topic_data
                .pages
                .pages
                .iter()
                .map(|(page_id, page)| {
                    let metrics = page.get_page_size_metrics();
                    TopicPageJsonContract {
                        id: page_id.into(),
                        amount: metrics.messages_amount,
                        size: metrics.data_size,
                        persist_size: metrics.persist_size,
                        sub_pages: page.get_sub_pages(),
                    }
                })
                .collect(),
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
