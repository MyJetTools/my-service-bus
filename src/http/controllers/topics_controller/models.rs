use crate::topics::Topic;

use my_http_server::macros::{MyHttpInput, MyHttpObjectStructure};
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
#[serde(transparent)]
pub struct JsonTopicsResult {
    pub items: Vec<JsonTopicResult>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct JsonTopicResult {
    pub id: String,
    #[serde(rename = "messageId")]
    pub message_id: i64,
}

impl JsonTopicResult {
    pub async fn new(topic: &Topic) -> Self {
        Self {
            id: topic.topic_id.to_string(),
            message_id: topic.get_message_id().await.into(),
        }
    }
}

#[derive(Debug, MyHttpInput)]
pub struct CreateTopicRequestContract {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,
}

#[derive(Debug, MyHttpInput)]
pub struct DeleteTopicRequestContract {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,
    #[http_query(name = "hardDeleteMoment"; description = "Moment when all data is going to be deleted forever")]
    pub hard_delete_moment: DateTimeAsMicroseconds,
}

#[derive(Debug, MyHttpInput)]
pub struct RestoreTopicRequestContract {
    #[http_query(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,
}

#[derive(Debug, MyHttpObjectStructure, Serialize)]
pub struct RestoreTopicResponseContract {
    pub restored: bool,
}

#[derive(Debug, MyHttpInput)]
pub struct UpdatePersistRequestContract {
    #[http_body(name = "topicId"; description = "Id of topic")]
    pub topic_id: String,
    #[http_body(description = "Persist or not persist")]
    pub persist: bool,
}
