use rust_extensions::base64::IntoBase64;

use crate::http::controllers::{MessageKeyValueJsonModel, MessageToDeliverHttpContract};
use crate::messages_page::MySbMessageContent;

pub struct SubscriberHttpPackageBuilder {
    messages: Vec<MessageToDeliverHttpContract>,
    data_size: usize,
}

impl SubscriberHttpPackageBuilder {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            data_size: 0,
        }
    }

    pub fn get_data_size(&self) -> usize {
        self.data_size
    }

    pub fn add_message(&mut self, msg: &MySbMessageContent, attempt_no: i32) {
        let msg_to_insert = MessageToDeliverHttpContract {
            id: msg.id.get_value(),
            headers: msg
                .headers
                .iter()
                .map(|(k, v)| MessageKeyValueJsonModel {
                    key: k.to_string(),
                    value: v.to_string(),
                })
                .collect(),
            attempt_no,
            content: msg.content.into_base64(),
        };

        self.data_size += msg_to_insert.content.as_bytes().len();
    }

    pub fn get_result(self) -> Vec<MessageToDeliverHttpContract> {
        self.messages
    }
}
