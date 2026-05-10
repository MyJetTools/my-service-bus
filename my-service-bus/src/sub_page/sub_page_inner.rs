use std::collections::BTreeMap;

use my_service_bus::abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus::shared::sub_page::SizeAndAmount;
use rust_extensions::sorted_vec::SortedVec;

use crate::messages_page::*;

pub enum GetMessageResult<'s> {
    Message(&'s MySbMessageContent),
    Missing,
    NotLoaded,
}

impl<'s> GetMessageResult<'s> {
    #[cfg(test)]
    pub fn is_not_loaded(&self) -> bool {
        match self {
            GetMessageResult::NotLoaded => true,
            _ => false,
        }
    }
}

pub struct SubPageInner {
    pub messages: SortedVec<MessageId, MySbCachedMessage>,
    size_and_amount: SizeAndAmount,
    to_persist: QueueWithIntervals,
}

impl SubPageInner {
    pub fn new() -> Self {
        Self {
            messages: SortedVec::new(),
            size_and_amount: SizeAndAmount::new(),
            to_persist: QueueWithIntervals::new(),
        }
    }

    pub fn restore(src: BTreeMap<i64, MySbCachedMessage>) -> Self {
        let mut size_and_amount = SizeAndAmount::new();

        let mut messages = SortedVec::new_with_capacity(src.len());

        for (_, msg) in src {
            if let MySbCachedMessage::Loaded(msg) = &msg {
                size_and_amount.added(msg.content.len());
            }
            messages.insert_or_replace(msg);
        }

        Self {
            messages,
            size_and_amount,
            to_persist: QueueWithIntervals::new(),
        }
    }

    pub fn get_size_metrics(&self) -> SizeMetrics {
        SizeMetrics {
            messages_amount: self.messages.len(),
            data_size: self.size_and_amount.size,
            persist_size: self.to_persist.queue_size(),
            avg_message_size: if self.messages.len() == 0 {
                0
            } else {
                self.size_and_amount.size / self.messages.len()
            },
        }
    }

    #[cfg(test)]
    pub fn get_all_messages_as_vec(&self) -> Vec<MySbMessageContent> {
        let mut result = Vec::new();

        for itm in self.messages.iter() {
            match itm {
                MySbCachedMessage::Loaded(content) => {
                    result.push(content.clone());
                }
                MySbCachedMessage::Missing(_) => {}
            }
        }

        result
    }

    pub fn add_message(
        &mut self,
        message: MySbCachedMessage,
        persist: bool,
    ) -> Option<MySbCachedMessage> {
        let message_id = message.get_message_id();

        self.size_and_amount.added(message.get_content_size());

        if persist {
            self.to_persist.enqueue(message_id.get_value());
        }

        let (_, old_message) = self.messages.insert_or_replace(message);

        if let Some(old_message) = old_message {
            self.size_and_amount.removed(old_message.get_content_size());
            return Some(old_message);
        }

        None
    }

    pub fn get_message<'s>(&'s self, msg_id: MessageId) -> GetMessageResult<'s> {
        if let Some(msg) = self.messages.get(&msg_id) {
            match msg {
                MySbCachedMessage::Loaded(msg) => GetMessageResult::Message(msg),
                MySbCachedMessage::Missing(_) => GetMessageResult::Missing,
            }
        } else {
            return GetMessageResult::NotLoaded;
        }
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) {
        let mut to_gc = Vec::new();
        for msg in self.messages.iter() {
            let msg_id = msg.get_message_id();
            if self.message_can_be_gc(msg_id, min_message_id) {
                to_gc.push(msg_id);
            } else {
                break;
            }
        }

        for msg_to_gc in to_gc {
            self.remove_message(msg_to_gc);
        }
    }

    fn remove_message(&mut self, msg_id: MessageId) -> usize {
        if let Some(msg) = self.messages.remove(&msg_id) {
            let freed_size = msg.get_content_size();
            self.size_and_amount.removed(freed_size);
            return freed_size;
        }

        0
    }

    pub fn get_messages_to_persist<TResult>(
        &self,
        transform: impl Fn(&MySbMessageContent) -> TResult,
    ) -> Option<Vec<TResult>> {
        if self.to_persist.queue_size() == 0 {
            return None;
        }

        let mut result = Vec::with_capacity(self.get_messages_to_persist_amount());
        for message_id in &self.to_persist {
            let message_id = MessageId::new(message_id);
            if let Some(msg) = self.messages.get(&message_id) {
                if let MySbCachedMessage::Loaded(msg) = msg {
                    result.push(transform(msg));
                }
            }
        }

        Some(result)
    }

    pub fn get_messages_to_persist_amount(&self) -> usize {
        self.to_persist.queue_size()
    }

    pub fn mark_messages_as_persisted(&mut self, ids: &QueueWithIntervals) {
        for message_id in ids {
            let _ = self.to_persist.remove(message_id);
        }
    }

    pub fn has_messages_to_persist(&self) -> bool {
        self.to_persist.queue_size() > 0
    }

    fn message_can_be_gc(&self, msg_id: MessageId, min_message_id: MessageId) -> bool {
        if self.to_persist.has_message(msg_id.get_value()) {
            return false;
        }

        msg_id.get_value() < min_message_id.get_value()
    }
}
