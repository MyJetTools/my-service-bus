use crate::topics::Topic;

use my_http_server_swagger::{MyHttpInput, MyHttpObjectStructure};
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
