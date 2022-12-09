use std::collections::HashMap;

use my_http_server::{HttpFailResult, WebContentType};

use my_http_server_swagger::{MyHttpInput, MyHttpObjectStructure};
use serde::{Deserialize, Serialize};

#[derive(MyHttpInput)]
pub struct PublishMessageHttpInput {
    #[http_header(description = "Http session")]
    pub authorization: String,

    #[http_query(name="topicId"; description = "Id of topic")]
    pub topic_id: String,

    #[http_body(description = "Base64 encoded messages")]
    pub messages: Vec<MessageToPublishJsonModel>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct MessageKeyValueJsonModel {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct MessageToPublishJsonModel {
    pub headers: Option<Vec<MessageKeyValueJsonModel>>,
    #[serde(rename = "base64Message")]
    pub base64_message: String,
}

impl MessageToPublishJsonModel {
    pub fn get_headers(&mut self) -> Option<HashMap<String, String>> {
        let mut result = None;

        std::mem::swap(&mut self.headers, &mut result);

        let src = result?;
        let mut result = HashMap::new();

        for itm in src {
            result.insert(itm.key, itm.value);
        }

        Some(result)
    }

    pub fn get_content(&self) -> Result<Vec<u8>, HttpFailResult> {
        match base64::decode(self.base64_message.as_str()) {
            Ok(bytes) => Ok(bytes),
            Err(err) => Err(HttpFailResult {
                content_type: WebContentType::Text,
                status_code: 400,
                content: format!("Can not convert content from Base64. Err: Err{}", err)
                    .into_bytes(),
                write_telemetry: false,
                write_to_log: false,
            }),
        }
    }
}
