use std::collections::BTreeMap;

use my_service_bus_abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus_shared::sub_page::{SizeAndAmount, SubPageId};
use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};

use super::{MessagesToPersistBucket, MySbCachedMessage, MySbMessageContent, SizeMetrics};

pub enum GetMessageResult<'s> {
    Message(&'s MySbMessageContent),
    Missing,
    GarbageCollected,
}

impl<'s> GetMessageResult<'s> {
    #[cfg(test)]
    pub fn is_message_content(&self) -> bool {
        match self {
            GetMessageResult::Message(_) => true,
            _ => false,
        }
    }
    #[cfg(test)]
    pub fn is_garbage_collected(&self) -> bool {
        match self {
            GetMessageResult::GarbageCollected => true,
            _ => false,
        }
    }
}

pub struct SubPageInner {
    pub sub_page_id: SubPageId,
    pub messages: BTreeMap<i64, MySbCachedMessage>,
    pub created: DateTimeAsMicroseconds,
    pub last_accessed: AtomicDateTimeAsMicroseconds,
    size_and_amount: SizeAndAmount,
    to_persist: QueueWithIntervals,
}

impl SubPageInner {
    pub fn new(sub_page_id: SubPageId) -> Self {
        let created = DateTimeAsMicroseconds::now();
        Self {
            sub_page_id,
            messages: BTreeMap::new(),
            created,
            size_and_amount: SizeAndAmount::new(),
            to_persist: QueueWithIntervals::new(),
            last_accessed: AtomicDateTimeAsMicroseconds::new(created.unix_microseconds),
        }
    }

    pub fn restore(sub_page_id: SubPageId, messages: BTreeMap<i64, MySbCachedMessage>) -> Self {
        let mut size_and_amount = SizeAndAmount::new();

        for msg in messages.values() {
            if let MySbCachedMessage::Loaded(msg) = msg {
                size_and_amount.added(msg.content.len());
            }
        }
        let created = DateTimeAsMicroseconds::now();
        Self {
            sub_page_id,
            messages,
            created,
            size_and_amount,
            to_persist: QueueWithIntervals::new(),
            last_accessed: AtomicDateTimeAsMicroseconds::new(created.unix_microseconds),
        }
    }

    pub fn get_size_metrics(&self) -> SizeMetrics {
        SizeMetrics {
            messages_amount: self.messages.len(),
            data_size: self.size_and_amount.size,
            persist_size: self.to_persist.len() as usize,
        }
    }

    pub fn update_last_accessed(&self, now: DateTimeAsMicroseconds) {
        self.last_accessed.update(now);
    }

    pub fn add_message(&mut self, message: MySbCachedMessage) -> Option<MySbCachedMessage> {
        let message_id = message.get_message_id();

        if !self.sub_page_id.is_my_message_id(message_id) {
            println!(
                "Somehow we are uploading message_id {} to sub_page {}. Skipping message...",
                message_id.get_value(),
                self.sub_page_id.get_value()
            );
            return None;
        }

        self.size_and_amount.added(message.get_content_size());

        self.to_persist.enqueue(message_id.get_value());

        if let Some(old_message) = self.messages.insert(message_id.get_value(), message) {
            self.size_and_amount.removed(old_message.get_content_size());
            return Some(old_message);
        }

        None
    }

    pub fn get_message(&self, msg_id: MessageId) -> GetMessageResult {
        if let Some(msg) = self.messages.get(msg_id.as_ref()) {
            match msg {
                MySbCachedMessage::Loaded(msg) => GetMessageResult::Message(msg),
                MySbCachedMessage::Missing(_) => GetMessageResult::Missing,
            }
        } else {
            return GetMessageResult::GarbageCollected;
        }
    }

    fn get_first_message_id(&self) -> Option<MessageId> {
        for msg_id in self.messages.keys() {
            return Some(MessageId::new(*msg_id));
        }

        None
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) -> bool {
        let first_message_id_of_next_page =
            self.sub_page_id.get_first_message_id_of_next_sub_page();

        if min_message_id.get_value() >= first_message_id_of_next_page.get_value() {
            return true;
        }

        if min_message_id.get_value() < self.sub_page_id.get_first_message_id().get_value() {
            return false;
        }

        let first_message_id = self.get_first_message_id();

        if first_message_id.is_none() {
            return false;
        }

        let first_message_id = first_message_id.unwrap();

        for msg_id in first_message_id.get_value()..min_message_id.get_value() {
            if let Some(message) = self.messages.remove(&msg_id) {
                self.size_and_amount.removed(message.get_content_size());
            }
        }

        false
    }

    pub fn get_messages_to_persist(&self, max_size: usize) -> Option<MessagesToPersistBucket> {
        if self.to_persist.len() == 0 {
            return None;
        }

        let mut result = MessagesToPersistBucket::new(self.sub_page_id);

        for message_id in &self.to_persist {
            if result.size >= max_size {
                break;
            }

            if let Some(msg) = self.messages.get(&message_id) {
                if let MySbCachedMessage::Loaded(msg) = msg {
                    result.add(msg.into());
                }
            }
        }

        result.into()
    }

    pub fn mark_messages_as_persisted(&mut self, ids: &QueueWithIntervals) {
        for message_id in ids {
            let _ = self.to_persist.remove(message_id);
        }
    }

    pub fn get_min_message_to_persist(&self) -> Option<MessageId> {
        let message_id = self.to_persist.peek()?;
        Some(message_id.into())
    }

    pub fn has_messages_to_persist(&self) -> bool {
        self.to_persist.len() > 0
    }
}

#[cfg(test)]
mod tests {

    use crate::messages_page::MySbMessageContent;

    use super::*;

    #[test]
    fn test_gc_messages() {
        let mut sub_page = SubPageInner::new(SubPageId::new(0));

        sub_page.add_message(
            MySbMessageContent {
                id: 0.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

        sub_page.add_message(
            MySbMessageContent {
                id: 1.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

        let gc_full_page = sub_page.gc_messages(1.into());

        assert!(!gc_full_page);

        let result = sub_page.get_message(0.into());
        assert!(result.is_garbage_collected());

        let result = sub_page.get_message(1.into());
        assert!(result.is_message_content());
    }

    #[test]
    pub fn test_gc_messages_prev_page() {
        let mut sub_page = SubPageInner::new(SubPageId::new(1));

        sub_page.add_message(
            MySbMessageContent {
                id: 1000.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

        sub_page.add_message(
            MySbMessageContent {
                id: 1001.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

        let gc_full_page = sub_page.gc_messages(5.into());

        assert!(!gc_full_page);

        let result = sub_page.get_message(1000.into());
        assert!(result.is_message_content());
    }

    #[test]
    pub fn test_gc_messages_next_page() {
        let mut sub_page = SubPageInner::new(SubPageId::new(1));

        sub_page.add_message(
            MySbMessageContent {
                id: 1000.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

        sub_page.add_message(
            MySbMessageContent {
                id: 1001.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            }
            .into(),
        );

        let gc_full_page = sub_page.gc_messages(9999.into());

        assert!(gc_full_page);

        let result = sub_page.get_message(1000.into());
        assert!(result.is_message_content());

        let result = sub_page.get_message(1001.into());
        assert!(result.is_message_content());
    }
}
