use my_http_server::macros::{MyHttpInput, MyHttpObjectStructure};
use serde::*;

#[derive(Debug, MyHttpInput)]
pub struct GetMinMessageIdInputModel {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct MinMessageIdDebugModel {
    pub min_message_id: Option<i64>,
}

/*
#[derive(Debug, MyHttpInput)]
pub struct EnableDebugInputModel {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,
    #[http_query(name = "queueId"; description = "Id of queue")]
    pub queue_id: String,
}
 */

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
    pub last_delivered_amount: usize,
    pub delivery_compilation: String,
}

#[derive(Debug, MyHttpInput)]
pub struct SetDebugConsoleTargetInputModel {
    #[http_query(
        name = "topicId";
        description = "Topic to trace. Leave empty to turn the debug console OFF"
    )]
    pub topic_id: Option<String>,
    #[http_query(
        name = "queueId";
        description = "Queue to trace. Leave empty to trace every queue of the topic"
    )]
    pub queue_id: Option<String>,
}

#[derive(Debug, MyHttpInput)]
pub struct GetDebugConsoleInputModel {
    #[http_query(name = "tail"; description = "Return only the last N records")]
    pub tail: Option<i64>,
    #[http_query(name = "clear"; description = "Clear the buffer after reading")]
    pub clear: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct DebugConsoleRecordHttpModel {
    pub date: String,
    pub data: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct DebugConsoleHttpModel {
    pub enabled: bool,
    pub topic_id: Option<String>,
    pub queue_id: Option<String>,
    pub records_count: usize,
    pub records: Vec<DebugConsoleRecordHttpModel>,
}
