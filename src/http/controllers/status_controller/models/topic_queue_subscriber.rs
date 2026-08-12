use my_http_server::macros::MyHttpObjectStructure;
use serde::{Deserialize, Serialize};

use crate::queue_subscribers::QueueSubscriber;

use super::queue_model::QueueIndex;

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
    #[serde(rename = "deliveryStateStr")]
    pub delivery_state_str: String,
    pub history: Vec<i32>,
    /// Message ids handed to the subscriber and still waiting for a confirmation.
    /// Empty unless the subscriber is in the `OnDelivery` state.
    #[serde(rename = "onDelivery")]
    pub on_delivery: Vec<QueueIndex>,
    #[serde(rename = "onDeliveryAmount")]
    pub on_delivery_amount: usize,
    /// Message ids of the current delivery the subscriber has already confirmed
    /// (intermediary confirmations) - they are gone from `onDelivery` by now.
    #[serde(rename = "confirmed")]
    pub confirmed: Vec<QueueIndex>,
    #[serde(rename = "confirmedAmount")]
    pub confirmed_amount: usize,
}

impl TopicQueueSubscriberJsonModel {
    pub fn new(subscriber: &QueueSubscriber) -> Self {
        let (on_delivery, on_delivery_amount, confirmed, confirmed_amount) =
            match subscriber.get_delivery_bucket() {
                Some(bucket) => (
                    QueueIndex::from_queue_with_intervals(&bucket.to_be_confirmed),
                    bucket.to_be_confirmed.queue_size(),
                    QueueIndex::from_queue_with_intervals(&bucket.confirmed),
                    bucket.confirmed.queue_size(),
                ),
                None => (Vec::new(), 0, Vec::new(), 0),
            };

        Self {
            subscriber_id: subscriber.id.get_value(),
            session_id: subscriber.session.session_id.get_value(),
            queue_id: subscriber.queue_id.to_string(),
            active: subscriber.metrics.active,
            delivery_state: subscriber.delivery_state.to_u8(),
            delivery_state_str: subscriber.delivery_state.as_str().to_string(),
            history: subscriber.metrics.delivery_history.get(),
            on_delivery,
            on_delivery_amount,
            confirmed,
            confirmed_amount,
        }
    }
}
