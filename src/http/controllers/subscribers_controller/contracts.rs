use my_http_server::macros::*;
use my_service_bus::abstractions::subscriber::TopicQueueType;
use serde_derive::{Deserialize, Serialize};

use crate::http::controllers::MessageToDeliverHttpContract;

#[derive(MyHttpStringEnum)]
pub enum QueueTypeHttpModel {
    #[http_enum_case(id: 0, description: "Queue is automatically deleted if it has no subscribers")]
    DeleteIfNoSubscribers,
    #[http_enum_case(id: 1, description: "Queue is never deleted")]
    Permanent,
    #[http_enum_case(id: 2, description: "Queue can have only a single subscriber")]
    PermanentSingleSubscriber,
}

#[derive(MyHttpInput)]
pub struct SubscribeHttpInputModel {
    #[http_body(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,

    #[http_body(name = "queueId"; description = "Id of queue")]
    pub queue_id: String,

    #[http_body(name = "queueType"; description = "Type of queue")]
    pub queue_type: QueueTypeHttpModel,
}

impl SubscribeHttpInputModel {
    pub fn get_queue_type(&self) -> TopicQueueType {
        match self.queue_type {
            QueueTypeHttpModel::DeleteIfNoSubscribers => TopicQueueType::DeleteOnDisconnect,
            QueueTypeHttpModel::Permanent => TopicQueueType::Permanent,
            QueueTypeHttpModel::PermanentSingleSubscriber => {
                TopicQueueType::PermanentWithSingleConnection
            }
        }
    }
}

#[derive(MyHttpInput)]
pub struct ConfirmDeliveryHttpModel {
    #[http_body(name = "confirmation"; description = "If something should be confirmed")]
    pub confirmation: Option<Vec<ConfirmationInfo>>,
}

#[derive(MyHttpInputObjectStructure, Serialize, Deserialize)]
pub struct ConfirmationInfo {
    #[serde(rename = "topicId")]
    pub topic_id: String,
    #[serde(rename = "queueId")]
    pub queue_id: String,
    #[serde(rename = "subscriberId")]
    pub subscriber_id: i64,
    #[serde(rename = "allOk")]
    pub all_is_ok: Option<bool>,
    #[serde(rename = "allFail")]
    pub all_is_fail: Option<bool>,
    #[serde(rename = "someOk")]
    pub ok_messages: Vec<QueueInterval>,
}

impl ConfirmationInfo {
    pub fn all_confirmed_ok(&self) -> bool {
        match self.all_is_ok {
            Some(result) => result,
            _ => false,
        }
    }
}

#[derive(MyHttpInputObjectStructure, Serialize, Deserialize)]
pub struct QueueInterval {
    #[serde(rename = "fromId")]
    pub from_id: String,
    #[serde(rename = "toId")]
    pub to_id: String,
}

#[derive(MyHttpObjectStructure, Serialize)]
pub struct AwaitDeliveryHttpResponse {
    pub topic_id: String,
    pub queue_id: String,
    pub confirmation_id: i64,
    pub messages: Vec<MessageToDeliverHttpContract>,
}
