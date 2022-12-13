use std::collections::{BTreeMap, HashMap};

use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::{protobuf_models::MessageProtobufModel, MySbMessageContent};
use tokio::sync::Mutex;

use super::PersistenceError;

pub struct MessagesPagesMockRepo {
    messages: Mutex<HashMap<String, HashMap<MessageId, MySbMessageContent>>>,
}

impl MessagesPagesMockRepo {
    pub fn new() -> Self {
        Self {
            messages: Mutex::new(HashMap::new()),
        }
    }

    pub async fn load_page(
        &self,
        topic_id: &str,
        from_message_id: MessageId,
        to_message_id: MessageId,
    ) -> Result<BTreeMap<i64, MySbMessageContent>, PersistenceError> {
        let mut result = BTreeMap::new();

        let mut write_access = self.messages.lock().await;

        if !write_access.contains_key(topic_id) {
            write_access.insert(topic_id.to_string(), HashMap::new());
        }

        let messages = write_access.get(topic_id).unwrap();

        for message_id in from_message_id..=to_message_id {
            if let Some(message) = messages.get(&message_id) {
                result.insert(message_id, message.clone());
            }
        }
        Ok(result)
    }

    pub async fn save_messages(
        &self,
        topic_id: &str,
        messages: Vec<MessageProtobufModel>,
    ) -> Result<(), PersistenceError> {
        let mut write_access = self.messages.lock().await;
        if !write_access.contains_key(topic_id) {
            write_access.insert(topic_id.to_string(), HashMap::new());
        }

        let messages_by_topic = write_access.get_mut(topic_id).unwrap();

        for message in messages {
            let model_to_save: MySbMessageContent = message.into();
            messages_by_topic.insert(model_to_save.id, model_to_save);
        }

        Ok(())
    }
}
