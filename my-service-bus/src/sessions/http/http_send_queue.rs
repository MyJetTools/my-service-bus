use rust_extensions::auto_shrink::VecAutoShrink;

use crate::queue_subscribers::SubscriberId;

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
        topic_id: &str,
        queue_id: &str,
        subscriber_id: SubscriberId,
    ) {
        for itm in self.queue.iter() {
            if itm.topic_id.as_str() == topic_id
                && itm.queue_id.as_str() == queue_id
                && itm.subscriber_id.get_value() == subscriber_id.get_value()
            {
                panic!(
                    "Messages already in the queue {}/{}. Subscriber:{}",
                    topic_id, queue_id, itm.subscriber_id
                );
            }
        }
    }

    pub fn enqueue_messages(&mut self, data: HttpDeliveryPackage) {
        self.check_if_we_have_it_already(&data.topic_id, &data.queue_id, data.subscriber_id);
        self.queue.push(data)
    }

    pub fn get_next_package(&mut self) -> Option<HttpDeliveryPackage> {
        if self.queue.len() == 0 {
            return None;
        }
        Some(self.queue.remove(0))
    }
}
