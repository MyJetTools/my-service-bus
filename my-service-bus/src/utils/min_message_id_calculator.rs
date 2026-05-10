use my_service_bus::abstractions::MessageId;

pub struct MinMessageIdCalculator {
    value: Option<i64>,
}

impl MinMessageIdCalculator {
    pub fn new() -> Self {
        Self { value: None }
    }

    pub fn add(&mut self, new_value: Option<impl Into<MessageId>>) {
        if let Some(new_value) = new_value {
            if let Some(value) = self.value {
                let new_value = new_value.into().get_value();
                if new_value < value {
                    self.value = Some(new_value);
                }
            } else {
                self.value = Some(new_value.into().get_value());
            }
        }
    }

    pub fn get(self) -> Option<MessageId> {
        let result = self.value?;
        Some(result.into())
    }
}
