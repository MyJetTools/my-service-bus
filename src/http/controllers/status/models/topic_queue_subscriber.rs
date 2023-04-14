use my_http_server_swagger::MyHttpObjectStructure;
use serde::{Deserialize, Serialize};

use crate::queue_subscribers::QueueSubscriber;

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct TopicQueueSubscriberJsonModel {
    #[serde(rename = "id")]
    pub subscriber_id: i64,
    #[serde(rename = "sessionId")]
    pub session_id: i64,
    #[serde(rename = "queueId")]
    pub queue_id: String,
    pub active: u8,
    #[serde(rename = "deliveryState")]
    pub delivery_state: u8,
    pub history: Vec<i32>,
}

impl TopicQueueSubscriberJsonModel {
    pub fn new(subscriber: &QueueSubscriber) -> Self {
        Self {
            subscriber_id: subscriber.id.get_value(),
            session_id: subscriber.session.id,
            queue_id: subscriber.queue_id.to_string(),
            active: subscriber.metrics.active,
            delivery_state: subscriber.delivery_state.to_u8(),
            history: subscriber.metrics.delivery_history.get(),
        }
    }
}
