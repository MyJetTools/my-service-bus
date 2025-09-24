use my_service_bus::abstractions::queue_with_intervals::QueueIndexRange;
use my_service_bus::abstractions::subscriber::TopicQueueType;
use my_service_bus::abstractions::SbMessageHeaders;
use my_service_bus::shared::protobuf_models::MessageProtobufModel;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::messages_page::MySbMessageContent;
use crate::topics::{TopicQueueSnapshot, TopicSnapshot};

use crate::persistence_grpc::*;

impl Into<MySbMessageContent> for MessageContentGrpcModel {
    fn into(self) -> MySbMessageContent {
        MySbMessageContent {
            id: self.message_id.into(),
            content: self.data,
            time: DateTimeAsMicroseconds::new(self.created),
            headers: to_message_headers(self.meta_data),
        }
    }
}

fn to_message_headers(src: Vec<MessageContentMetaDataItem>) -> SbMessageHeaders {
    let mut result = SbMessageHeaders::with_capacity(src.len());

    for itm in src {
        result.add_header(itm.key, itm.value);
    }

    result
}

impl From<&TopicSnapshot> for TopicAndQueuesSnapshotGrpcModel {
    fn from(src: &TopicSnapshot) -> Self {
        Self {
            topic_id: src.topic_id.to_string(),
            message_id: src.message_id,
            queue_snapshots: src.queues.iter().map(|itm| itm.into()).collect(),
            persist: Some(src.persist),
        }
    }
}

impl From<TopicAndQueuesSnapshotGrpcModel> for TopicSnapshot {
    fn from(src: TopicAndQueuesSnapshotGrpcModel) -> Self {
        Self {
            topic_id: src.topic_id.as_str().into(),
            message_id: src.message_id,
            persist: if let Some(persist) = src.persist {
                persist
            } else {
                true
            },
            queues: src
                .queue_snapshots
                .into_iter()
                .map(|itm| itm.into())
                .collect(),
        }
    }
}

impl From<&TopicQueueSnapshot> for QueueSnapshotGrpcModel {
    fn from(src: &TopicQueueSnapshot) -> Self {
        Self {
            queue_id: src.queue_id.to_string(),
            queue_type: src.queue_type.into_u8() as i32,
            ranges: src.ranges.iter().map(|itm| itm.into()).collect(),
        }
    }
}

impl From<QueueSnapshotGrpcModel> for TopicQueueSnapshot {
    fn from(src: QueueSnapshotGrpcModel) -> Self {
        Self {
            queue_id: src.queue_id.to_string(),
            queue_type: TopicQueueType::from_u8(src.queue_type as u8),
            ranges: src.ranges.into_iter().map(|itm| itm.into()).collect(),
        }
    }
}

impl From<&QueueIndexRange> for QueueIndexRangeGrpcModel {
    fn from(src: &QueueIndexRange) -> Self {
        Self {
            from_id: src.from_id,
            to_id: src.to_id,
        }
    }
}

impl From<QueueIndexRangeGrpcModel> for QueueIndexRange {
    fn from(src: QueueIndexRangeGrpcModel) -> Self {
        Self {
            from_id: src.from_id,
            to_id: src.to_id,
        }
    }
}

impl Into<MessageContentGrpcModel> for MessageProtobufModel {
    fn into(self) -> MessageContentGrpcModel {
        MessageContentGrpcModel {
            message_id: self.get_message_id().get_value(),
            created: self.get_created().unix_microseconds,
            data: self.data,
            meta_data: self.headers.into_iter().map(|itm| itm.into()).collect(),
        }
    }
}

impl Into<MessageContentMetaDataItem>
    for my_service_bus::shared::protobuf_models::MessageMetaDataProtobufModel
{
    fn into(self) -> MessageContentMetaDataItem {
        MessageContentMetaDataItem {
            key: self.key,
            value: self.value,
        }
    }
}

/*
impl Into<TopicSnapshot> for TopicAndQueuesSnapshotGrpcModel {
    fn into(self) -> TopicSnapshot {
        TopicSnapshot {
            topic_id: ShortString::from_str(self.topic_id.as_str()).unwrap(),
            message_id: self.message_id,
            queues: self
                .queue_snapshots
                .into_iter()
                .map(|itm| TopicQueueSnapshot {
                    queue_id: itm.queue_id,
                    queue_type: TopicQueueType::from_u8(itm.queue_type as u8),
                    ranges: itm
                        .ranges
                        .into_iter()
                        .map(|itm| QueueIndexRange {
                            from_id: itm.from_id,
                            to_id: itm.to_id,
                        })
                        .collect(),
                }),
            persist: self.persist(),
        }
    }
}
 */
