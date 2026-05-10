use crate::{http::controllers::MessageToDeliverHttpContract, queue_subscribers::SubscriberId};

pub struct HttpDeliveryPackage {
    pub topic_id: String,
    pub queue_id: String,
    pub subscriber_id: SubscriberId,
    pub messages: Vec<MessageToDeliverHttpContract>,
}
