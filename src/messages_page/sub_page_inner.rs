use std::collections::BTreeMap;

use my_service_bus::abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus::shared::sub_page::{SizeAndAmount, SubPageId};
use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};

use super::{MySbCachedMessage, MySbMessageContent, SizeMetrics};

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
            persist_size: self.to_persist.queue_size(),
            avg_message_size: if self.messages.len() == 0 {
                0
            } else {
                self.size_and_amount.size / self.messages.len()
            },
        }
    }

    pub fn update_last_accessed(&self, now: DateTimeAsMicroseconds) {
        self.last_accessed.update(now);
    }

    pub fn add_message(
        &mut self,
        message: MySbCachedMessage,
        persist: bool,
    ) -> Option<MySbCachedMessage> {
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

        if persist {
            self.to_persist.enqueue(message_id.get_value());
        }

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

    pub fn gc_messages(&mut self, min_message_id: MessageId) {
        let mut to_gc = Vec::new();
        for msg_id in self.messages.keys() {
            if self.message_can_be_gc(*msg_id, min_message_id) {
                to_gc.push(*msg_id);
            } else {
                break;
            }
        }

        for msg_to_gc in to_gc {
            self.remove_message(msg_to_gc);
        }
    }

    fn remove_message(&mut self, msg_id: i64) -> usize {
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

    pub fn is_ready_to_gc(&self, min_message_id: MessageId) -> bool {
        if self.has_messages_to_persist() {
            return false;
        }

        let min_message_sub_page_id: SubPageId = min_message_id.into();

        if self.sub_page_id.get_value() < min_message_sub_page_id.get_value() {
            return true;
        }

        for msg_id in self.messages.keys() {
            if !self.message_can_be_gc(*msg_id, min_message_id) {
                return false;
            }
        }

        true
    }

    fn message_can_be_gc(&self, msg_id: i64, min_message_id: MessageId) -> bool {
        if self.to_persist.has_message(msg_id) {
            return false;
        }

        msg_id < min_message_id.get_value()
    }
}

#[cfg(test)]
mod tests {

    use my_service_bus::abstractions::SbMessageHeaders;

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
                headers: SbMessageHeaders::new(),
            }
            .into(),
            false,
        );

        sub_page.add_message(
            MySbMessageContent {
                id: 1.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: SbMessageHeaders::new(),
            }
            .into(),
            false,
        );

        let min_message_id = 1.into();

        sub_page.gc_messages(min_message_id);

        assert_eq!(sub_page.is_ready_to_gc(min_message_id), false);

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
                headers: SbMessageHeaders::new(),
            }
            .into(),
            true,
        );

        sub_page.add_message(
            MySbMessageContent {
                id: 1001.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: SbMessageHeaders::new(),
            }
            .into(),
            true,
        );

        let min_message_id = 5.into();
        sub_page.gc_messages(min_message_id);

        assert_eq!(sub_page.is_ready_to_gc(min_message_id), false);

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
                headers: SbMessageHeaders::new(),
            }
            .into(),
            true,
        );

        sub_page.add_message(
            MySbMessageContent {
                id: 1001.into(),
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: SbMessageHeaders::new(),
            }
            .into(),
            true,
        );

        let min_message_id = 9999.into();

        sub_page.gc_messages(min_message_id);

        assert_eq!(sub_page.is_ready_to_gc(min_message_id), false);

        let result = sub_page.get_message(1000.into());
        assert!(result.is_message_content());

        let result = sub_page.get_message(1001.into());
        assert!(result.is_message_content());
    }
}
