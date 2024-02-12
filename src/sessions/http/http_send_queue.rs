use rust_extensions::auto_shrink::VecAutoShrink;

use crate::{
    http::controllers::MessageToDeliverHttpContract, queue_subscribers::SubscriberId,
    queues::QueueId, topics::TopicId,
};

use super::HttpDeliveryPackage;

pub struct HttpSendQueue {
    queue: VecAutoShrink<HttpDeliveryPackage>,
}

impl HttpSendQueue {
    pub fn new() -> Self {
        Self {
            queue: VecAutoShrink::new(16),
        }
    }

    fn check_if_we_have_it_already(
        &self,
        topic_id: &TopicId,
        queue_id: &QueueId,
        subscriber_id: SubscriberId,
    ) {
        for itm in self.queue.iter() {
            if itm.topic_id.as_str() == topic_id.as_str()
                && itm.queue_id.as_str() == queue_id.as_str()
                && itm.subscriber_id.get_value() == subscriber_id.get_value()
            {
                panic!(
                    "Messages already in the queue {}/{}. Subscriber:{}",
                    topic_id.as_str(),
                    queue_id.as_str(),
                    itm.subscriber_id
                );
            }
        }
    }

    pub fn enqueue_messages(
        &mut self,
        topic_id: TopicId,
        queue_id: QueueId,
        subscriber_id: SubscriberId,
        messages: Vec<MessageToDeliverHttpContract>,
    ) {
        self.check_if_we_have_it_already(&topic_id, &queue_id, subscriber_id);

        self.queue.push(HttpDeliveryPackage {
            topic_id,
            queue_id,
            subscriber_id,
            messages,
        })
    }

    pub fn get_next_package(&mut self) -> Option<HttpDeliveryPackage> {
        if self.queue.len() == 0 {
            return None;
        }
        Some(self.queue.remove(0))
    }
}
