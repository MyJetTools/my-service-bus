use my_service_bus::shared::validators::InvalidTopicName;

use crate::queue_subscribers::SubscriberId;

#[derive(Debug)]
pub enum OperationFailResult {
    TopicNotFound { topic_id: String },
    QueueNotFound { queue_id: String },
    SubscriberNotFound { id: SubscriberId },
    SessionIsDisconnected,
    InvalidProtobufPayload(String),
    PersistenceError(String),
    TonicError(tonic::Status),
    Other(String),
    ShuttingDown,
    TopicOrQueueValidationError(InvalidTopicName),
}

impl From<InvalidTopicName> for OperationFailResult {
    fn from(src: InvalidTopicName) -> Self {
        Self::TopicOrQueueValidationError(src)
    }
}
