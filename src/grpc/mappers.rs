use my_service_bus_shared::queue_with_intervals::QueueIndexRange;

use crate::{
    queues::TopicQueueType,
    topics::{TopicQueueSnapshot, TopicSnapshot},
};

use crate::persistence_grpc::*;

impl From<Vec<TopicSnapshot>> for SaveQueueSnapshotGrpcRequest {
    fn from(src: Vec<TopicSnapshot>) -> Self {
        let queue_snapshot = src.iter().map(|itm| itm.into()).collect();
        Self { queue_snapshot }
    }
}

impl From<&TopicSnapshot> for TopicAndQueuesSnapshotGrpcModel {
    fn from(src: &TopicSnapshot) -> Self {
        Self {
            topic_id: src.topic_id.to_string(),
            message_id: src.message_id,
            queue_snapshots: src.queues.iter().map(|itm| itm.into()).collect(),
        }
    }
}

impl From<TopicAndQueuesSnapshotGrpcModel> for TopicSnapshot {
    fn from(src: TopicAndQueuesSnapshotGrpcModel) -> Self {
        Self {
            topic_id: src.topic_id,
            message_id: src.message_id,
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
            queue_type: src.queue_type.into(),
            ranges: src.ranges.iter().map(|itm| itm.into()).collect(),
        }
    }
}

impl From<QueueSnapshotGrpcModel> for TopicQueueSnapshot {
    fn from(src: QueueSnapshotGrpcModel) -> Self {
        Self {
            queue_id: src.queue_id.to_string(),
            queue_type: TopicQueueType::parse(src.queue_type as u8).unwrap(), //TDOD - remove unwrap
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

impl Into<i32> for TopicQueueType {
    fn into(self) -> i32 {
        let res: u8 = self.into();
        res as i32
    }
}
