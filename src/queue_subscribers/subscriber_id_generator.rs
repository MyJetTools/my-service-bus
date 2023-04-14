use std::sync::atomic::AtomicI64;

use super::SubscriberId;

pub struct SubscriberIdGenerator {
    current_id: AtomicI64,
}

impl SubscriberIdGenerator {
    pub fn new() -> Self {
        Self {
            current_id: AtomicI64::new(0),
        }
    }
    pub fn get_next_subscriber_id(&self) -> SubscriberId {
        let result = self
            .current_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        return result.into();
    }
}
