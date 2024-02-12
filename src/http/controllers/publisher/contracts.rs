use my_http_server::{HttpFailResult, WebContentType};

use my_http_server::macros::{MyHttpInput, MyHttpObjectStructure};
use my_service_bus::abstractions::SbMessageHeaders;
use rust_extensions::base64::FromBase64;
use serde::{Deserialize, Serialize};

use crate::http::controllers::MessageKeyValueJsonModel;

#[derive(MyHttpInput)]
pub struct PublishMessageHttpInput {
    #[http_query(name="topicId"; description = "Id of topic")]
    pub topic_id: String,

    #[http_body(description = "Base64 encoded messages")]
    pub messages: Vec<MessageToPublishJsonModel>,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct MessageToPublishJsonModel {
    pub headers: Option<Vec<MessageKeyValueJsonModel>>,
    #[serde(rename = "base64Message")]
    pub base64_message: String,
}

impl MessageToPublishJsonModel {
    pub fn get_headers(&mut self) -> SbMessageHeaders {
        if let Some(headers) = self.headers.take() {
            let mut result = SbMessageHeaders::with_capacity(headers.len());
            for itm in headers {
                result = result.add(itm.key, itm.value);
            }

            return result;
        }

        SbMessageHeaders::new()
    }

    pub fn get_content(&self) -> Result<Vec<u8>, HttpFailResult> {
        match self.base64_message.from_base64() {
            Ok(bytes) => Ok(bytes),
            Err(err) => Err(HttpFailResult {
                content_type: WebContentType::Text,
                status_code: 400,
                content: format!("Can not convert content from Base64. Err: {}", err).into_bytes(),
                write_telemetry: false,
                write_to_log: false,
            }),
        }
    }
}
