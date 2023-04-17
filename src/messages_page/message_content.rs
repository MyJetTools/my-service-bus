use std::collections::HashMap;

use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::protobuf_models::MessageProtobufModel;
use rust_extensions::date_time::DateTimeAsMicroseconds;

#[derive(Debug, Clone)]
pub struct MySbMessageContent {
    pub id: MessageId,
    pub content: Vec<u8>,
    pub time: DateTimeAsMicroseconds,
    pub headers: Option<HashMap<String, String>>,
}

impl MySbMessageContent {
    pub fn new(
        id: MessageId,
        content: Vec<u8>,
        headers: Option<HashMap<String, String>>,
        time: DateTimeAsMicroseconds,
    ) -> Self {
        Self {
            id,
            content,
            time,
            headers,
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            id: self.id,
            content: self.content.clone(),
            time: self.time,
            headers: self.headers.clone(),
        }
    }
}

impl Into<MessageProtobufModel> for &MySbMessageContent {
    fn into(self) -> MessageProtobufModel {
        MessageProtobufModel::new(
            self.id,
            self.time,
            self.content.clone(),
            convert_headers_from_hash_map(&self.headers),
        )
    }
}

impl From<MessageProtobufModel> for MySbMessageContent {
    fn from(src: MessageProtobufModel) -> Self {
        Self {
            id: src.get_message_id(),
            time: src.get_created(),
            content: src.data,
            headers: convert_headers_to_hash_map(src.headers),
        }
    }
}

fn convert_headers_to_hash_map(
    src: Vec<my_service_bus_shared::protobuf_models::MessageMetaDataProtobufModel>,
) -> Option<HashMap<String, String>> {
    if src.len() == 0 {
        return None;
    }

    let mut result = HashMap::new();

    for header in src {
        result.insert(header.key, header.value);
    }

    Some(result)
}

fn convert_headers_from_hash_map(
    src: &Option<HashMap<String, String>>,
) -> Vec<my_service_bus_shared::protobuf_models::MessageMetaDataProtobufModel> {
    if let Some(src) = src {
        let mut result = Vec::with_capacity(src.len());
        for (key, value) in src {
            result.push(
                my_service_bus_shared::protobuf_models::MessageMetaDataProtobufModel {
                    key: key.clone(),
                    value: value.clone(),
                },
            );
        }
        return result;
    }
    vec![]
}
