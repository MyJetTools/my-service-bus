use crate::messages_page::{MySbMessageContent, SizeMetrics};

use super::{GetMessageResult, SubPageInner};
use my_service_bus::abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus::shared::sub_page::SubPageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use rust_extensions::sorted_vec::EntityWithKey;

pub struct SubPage {
    pub id: SubPageId,
    pub last_accessed: DateTimeAsMicroseconds,
    inner: SubPageInner,
}

impl SubPage {
    pub fn new_as_brand_new(id: SubPageId) -> Self {
        Self {
            id,
            inner: SubPageInner::new(),
            last_accessed: DateTimeAsMicroseconds::now(),
        }
    }

    pub fn restore(id: SubPageId, inner: SubPageInner) -> Self {
        Self {
            id,
            inner,
            last_accessed: DateTimeAsMicroseconds::now(),
        }
    }

    pub fn get_message<'s>(&'s self, msg_id: MessageId) -> GetMessageResult<'s> {
        self.inner.get_message(msg_id)
    }

    pub fn update_last_accessed(&mut self, now: DateTimeAsMicroseconds) {
        self.last_accessed = now;
    }
    pub fn add_message(&mut self, msg: MySbMessageContent, persist: bool) {
        if !self.id.is_my_message_id(msg.id) {
            println!(
                "Somehow we are uploading message_id {} to sub_page {}. Skipping message...",
                msg.id.get_value(),
                self.id.get_value()
            );
            return;
        }

        self.inner.add_message(msg.into(), persist);
    }
    #[cfg(test)]
    pub fn unwrap_all_messages_with_content(&self) -> Vec<MySbMessageContent> {
        self.inner.get_all_messages_as_vec()
    }

    pub fn get_messages_to_persist<TResult>(
        &self,
        result: &mut Vec<(SubPageId, Vec<TResult>)>,
        transform: &impl Fn(&MySbMessageContent) -> TResult,
    ) {
        if let Some(messages_to_persist) = self.inner.get_messages_to_persist(transform) {
            result.push((self.id, messages_to_persist));
        }
    }

    pub fn mark_messages_as_persisted(&mut self, ids: &QueueWithIntervals) {
        self.inner.mark_messages_as_persisted(ids);
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) {
        self.inner.gc_messages(min_message_id);
    }

    pub fn is_empty(&self) -> bool {
        self.inner.messages.len() == 0
    }

    pub fn get_id(&self) -> SubPageId {
        self.id
    }

    pub fn get_size_metrics(&self) -> SizeMetrics {
        self.inner.get_size_metrics()
    }

    pub fn is_ready_to_gc(&self, active_pages: &[SubPageId]) -> bool {
        if self.inner.has_messages_to_persist() {
            return false;
        }

        for active_page in active_pages {
            if active_page.get_value() == self.id.get_value() {
                return false;
            }
        }

        true
    }
}

impl EntityWithKey<i64> for SubPage {
    fn get_key(&self) -> &i64 {
        self.id.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use my_service_bus::abstractions::SbMessageHeaders;

    impl MySbMessageContent {
        pub fn create_test_message(id: i64) -> Self {
            Self::new(
                id.into(),
                Vec::new(),
                SbMessageHeaders::new(),
                DateTimeAsMicroseconds::now(),
            )
        }
    }

    #[test]
    fn gc_removes_only_messages_below_min_id() {
        let mut sub_page = SubPage::new_as_brand_new(SubPageId::new(0));

        sub_page.add_message(MySbMessageContent::create_test_message(0), false);
        sub_page.add_message(MySbMessageContent::create_test_message(1), false);

        sub_page.gc_messages(1.into());

        assert!(matches!(
            sub_page.get_message(0.into()),
            GetMessageResult::NotLoaded
        ));
        assert!(matches!(
            sub_page.get_message(1.into()),
            GetMessageResult::Message(_)
        ));
    }

    #[test]
    fn is_not_ready_to_gc_while_persist_queue_has_items() {
        let mut sub_page = SubPage::new_as_brand_new(SubPageId::new(0));

        sub_page.add_message(MySbMessageContent::create_test_message(10), true);

        // Pending persistence must block GC even if there are no active pages provided.
        assert_eq!(sub_page.is_ready_to_gc(&[]), false);

        // Active pages also block GC.
        assert_eq!(sub_page.is_ready_to_gc(&[SubPageId::new(0)]), false);

        let mut persisted = QueueWithIntervals::new();
        persisted.enqueue(10);
        sub_page.mark_messages_as_persisted(&persisted);

        // Still active, so GC is blocked.
        assert_eq!(sub_page.is_ready_to_gc(&[SubPageId::new(0)]), false);

        // Once the page is not active and nothing is pending persistence, GC is allowed.
        assert_eq!(sub_page.is_ready_to_gc(&[]), true);
    }

    #[test]
    fn gc_does_not_remove_pending_persist_message() {
        let mut sub_page = SubPage::new_as_brand_new(SubPageId::new(0));

        sub_page.add_message(MySbMessageContent::create_test_message(0), true); // pending persistence
        sub_page.add_message(MySbMessageContent::create_test_message(1), false);

        // min_id is above both messages, but message 0 is still pending persist
        sub_page.gc_messages(2.into());

        assert!(matches!(
            sub_page.get_message(0.into()),
            GetMessageResult::Message(_)
        ));
        assert!(matches!(
            sub_page.get_message(1.into()),
            GetMessageResult::Message(_)
        ));

        let mut persisted = QueueWithIntervals::new();
        persisted.enqueue(0);
        sub_page.mark_messages_as_persisted(&persisted);

        // Now GC can clear the pending one as well
        sub_page.gc_messages(2.into());

        assert!(matches!(
            sub_page.get_message(0.into()),
            GetMessageResult::NotLoaded
        ));
        assert!(matches!(
            sub_page.get_message(1.into()),
            GetMessageResult::NotLoaded
        ));
    }

    #[test]
    fn gc_stops_when_first_message_is_protected_by_persist() {
        let mut sub_page = SubPage::new_as_brand_new(SubPageId::new(0));

        // First message is pending persist; second is clear
        sub_page.add_message(MySbMessageContent::create_test_message(0), true);
        sub_page.add_message(MySbMessageContent::create_test_message(1), false);

        // Even though min_id allows GC, the protected first message halts the sweep
        sub_page.gc_messages(2.into());

        assert!(matches!(
            sub_page.get_message(0.into()),
            GetMessageResult::Message(_)
        ));
        assert!(matches!(
            sub_page.get_message(1.into()),
            GetMessageResult::Message(_)
        ));
    }
}
