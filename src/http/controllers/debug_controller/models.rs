use my_http_server::macros::{MyHttpInput, MyHttpObjectStructure};
use serde::*;

#[derive(Debug, MyHttpInput)]
pub struct EnableDebugInputModel {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,
    #[http_query(name = "queueId"; description = "Id of queue")]
    pub queue_id: String,
}

#[derive(Debug, MyHttpInput)]
pub struct GetOnDeliveryInputModel {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,
    #[http_query(name = "queueId"; description = "Id of queue")]
    pub queue_id: String,
    #[http_query(name = "subscriberId"; description = "Id of subscriber")]
    pub subscriber_id: i64,
}

#[derive(Debug, MyHttpInput)]
pub struct GetQueuesAwaitingToDeliver {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct QueueDebugModel {
    pub name: String,
    pub queue_type: String,
    pub subscribers: Vec<QueueSubscriberDebugModel>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct QueueSubscriberDebugModel {
    pub id: i64,
    pub session_id: i64,
    pub subscribed: String,
    pub delivery_status: String,
    pub last_delivered: String,
}
