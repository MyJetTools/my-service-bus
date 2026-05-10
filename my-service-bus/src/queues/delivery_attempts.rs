use my_service_bus::abstractions::MessageId;
use rust_extensions::sorted_vec::{EntityWithKey, SortedVec};

pub struct DeliveryAttempt {
    pub message_id: MessageId,
    pub attempt: i32,
}

impl EntityWithKey<MessageId> for DeliveryAttempt {
    fn get_key(&self) -> &MessageId {
        &self.message_id
    }
}

pub struct DeliveryAttempts {
    attempts: SortedVec<MessageId, DeliveryAttempt>,
}

impl DeliveryAttempts {
    pub fn new() -> Self {
        Self {
            attempts: SortedVec::new(),
        }
    }

    pub fn get(&self, message_id: MessageId) -> i32 {
        if let Some(result) = self.attempts.get(&message_id) {
            result.attempt
        } else {
            0
        }
    }

    pub fn reset(&mut self, message_id: MessageId) {
        self.attempts.remove(&message_id);
    }

    pub fn add(&mut self, message_id: MessageId) {
        match self.attempts.insert_or_update(&message_id) {
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Insert(entry) => {
                entry.insert_and_get_index(DeliveryAttempt {
                    message_id,
                    attempt: 0,
                });
            }
            rust_extensions::sorted_vec::InsertOrUpdateEntry::Update(entry) => {
                entry.item.attempt += 1;
            }
        }
    }
}
