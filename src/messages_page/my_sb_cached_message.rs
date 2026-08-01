use my_service_bus::abstractions::MessageId;
use rust_extensions::sorted_vec::EntityWithKey;

use super::MySbMessageContent;

#[derive(Debug)]
pub enum MySbCachedMessage {
    Loaded(MySbMessageContent),
    Missing(MessageId),
}

impl MySbCachedMessage {
    pub fn get_content_size(&self) -> usize {
        match self {
            MySbCachedMessage::Loaded(msg) => msg.content.len(),

            MySbCachedMessage::Missing(_) => 0,
        }
    }

    pub fn get_message_id(&self) -> MessageId {
        match self {
            MySbCachedMessage::Loaded(msg) => msg.id,
            MySbCachedMessage::Missing(id) => *id,
        }
    }

    pub fn is_missing(&self) -> bool {
        match self {
            MySbCachedMessage::Loaded(_) => false,
            MySbCachedMessage::Missing(_) => true,
        }
    }

    pub fn unwrap_as_message(&self) -> &MySbMessageContent {
        match self {
            MySbCachedMessage::Loaded(msg) => msg,
            MySbCachedMessage::Missing(id) => panic!("Message {} is missing", id.get_value()),
        }
    }
}

impl Into<MySbCachedMessage> for MySbMessageContent {
    fn into(self) -> MySbCachedMessage {
        MySbCachedMessage::Loaded(self)
    }
}

impl EntityWithKey<MessageId> for MySbCachedMessage {
    fn get_key(&self) -> &MessageId {
        match self {
            MySbCachedMessage::Loaded(msg) => &msg.id,
            MySbCachedMessage::Missing(id) => id,
        }
    }
}
