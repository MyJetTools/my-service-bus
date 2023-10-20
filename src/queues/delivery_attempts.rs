use std::collections::HashMap;

use my_service_bus::abstractions::MessageId;

pub struct DeliveryAttempts {
    attempts: HashMap<MessageId, i32>,
}

impl DeliveryAttempts {
    pub fn new() -> Self {
        Self {
            attempts: HashMap::new(),
        }
    }

    pub fn get(&self, message_id: MessageId) -> i32 {
        if let Some(result) = self.attempts.get(&message_id) {
            *result
        } else {
            0
        }
    }

    pub fn reset(&mut self, message_id: MessageId) {
        self.attempts.remove(&message_id);
    }

    pub fn add(&mut self, message_id: MessageId) {
        let result = self.attempts.get(&message_id);

        match result {
            Some(value) => {
                let value = value.clone();
                self.attempts.insert(message_id, value + 1);
            }
            None => {
                self.attempts.insert(message_id, 0);
            }
        }
    }
}
