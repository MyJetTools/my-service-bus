use my_service_bus_abstractions::MessageId;

pub struct MinMessageIdCalculator {
    pub value: Option<MessageId>,
}

impl MinMessageIdCalculator {
    pub fn new() -> Self {
        Self { value: None }
    }

    pub fn add(&mut self, new_value: Option<MessageId>) {
        if let Some(new_value) = new_value {
            if let Some(value) = self.value {
                if new_value < value {
                    self.value = Some(new_value);
                }
            } else {
                self.value = Some(new_value);
            }
        }
    }
}
