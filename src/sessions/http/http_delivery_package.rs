use crate::{
    http::controllers::MessageToDeliverHttpContract, queue_subscribers::SubscriberId,
    queues::QueueId, topics::TopicId,
};

pub struct HttpDeliveryPackage {
    pub topic_id: TopicId,
    pub queue_id: QueueId,
    pub subscriber_id: SubscriberId,
    pub messages: Vec<MessageToDeliverHttpContract>,
}
