use my_service_bus::abstractions::publisher::SbMessageHeaders;
use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::protobuf_models::{MessageMetaDataProtobufModel, MessageProtobufModel};
use rust_extensions::date_time::DateTimeAsMicroseconds;

#[derive(Debug, Clone)]
pub struct MySbMessageContent {
    pub id: MessageId,
    pub content: Vec<u8>,
    pub time: DateTimeAsMicroseconds,
    pub headers: SbMessageHeaders,
}

impl MySbMessageContent {
    pub fn new(
        id: MessageId,
        content: Vec<u8>,
        headers: SbMessageHeaders,
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
            self.headers
                .iter()
                .map(|x| MessageMetaDataProtobufModel {
                    key: x.0.clone(),
                    value: x.1.clone(),
                })
                .collect(),
        )
    }
}

impl From<MessageProtobufModel> for MySbMessageContent {
    fn from(src: MessageProtobufModel) -> Self {
        Self {
            id: src.get_message_id(),
            time: src.get_created(),
            content: src.data,
            headers: SbMessageHeaders::from_iterator(
                src.headers.len().into(),
                src.headers.into_iter().map(|x| (x.key, x.value)),
            ),
        }
    }
}

//impl From<&Vec<MessageMetaDataProtobufModel>> for SbMessageHeaders {
/*
fn into(self) -> SbMessageHeaders {
    let mut result = SbMessageHeaders::new();

    for itm in self {
        result = result.add(itm.key.clone(), itm.value.clone());
    }

    result
}
 */
//}

/*
fn convert_headers_from_grpc(
    src: Vec<my_service_bus::shared::protobuf_models::MessageMetaDataProtobufModel>,
) -> Vec<(String, String)> {
    let mut result = Vec::with_capacity(src.len());
    for header in src {
        result.push((header.key, header.value));
    }
    result
}

fn convert_headers_to_grpc(
    src: &Vec<(String, String)>,
) -> Vec<my_service_bus::shared::protobuf_models::MessageMetaDataProtobufModel> {
    let mut result = Vec::with_capacity(src.len());
    for (key, value) in src {
        result.push(
            my_service_bus::shared::protobuf_models::MessageMetaDataProtobufModel {
                key: key.clone(),
                value: value.clone(),
            },
        );
    }
    result
}
 */
