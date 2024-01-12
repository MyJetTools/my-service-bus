use std::sync::Arc;

use super::{GetMessageResult, MissingSubPageInner, MySbMessageContent, SizeMetrics, SubPageInner};
use my_service_bus::abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus::shared::sub_page::SubPageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

pub enum SubPage {
    SubPage(SubPageInner),
    AllMessagesMissing(MissingSubPageInner),
}

impl SubPage {
    pub fn new(sub_page: SubPageInner) -> Self {
        Self::SubPage(sub_page)
    }

    pub fn create_as_missing(sub_page_id: SubPageId) -> Self {
        Self::AllMessagesMissing(MissingSubPageInner::new(sub_page_id))
    }

    pub fn get_id(&self) -> SubPageId {
        match self {
            SubPage::SubPage(inner) => inner.sub_page_id,
            SubPage::AllMessagesMissing(inner) => inner.sub_page_id,
        }
    }

    pub fn get_message(&self, msg_id: MessageId) -> GetMessageResult {
        match self {
            SubPage::SubPage(inner) => inner.get_message(msg_id),
            SubPage::AllMessagesMissing(_) => GetMessageResult::Missing,
        }
    }

    pub fn update_last_accessed(&self, now: DateTimeAsMicroseconds) {
        match self {
            SubPage::SubPage(inner) => inner.update_last_accessed(now),
            SubPage::AllMessagesMissing(inner) => inner.update_last_accessed(now),
        }
    }
    pub fn add_message(&mut self, msg: MySbMessageContent, persist: bool) {
        match self {
            SubPage::SubPage(inner) => {
                inner.add_message(msg.into(), persist);
            }
            SubPage::AllMessagesMissing(_) => {
                panic!("Trying to add message to archived missing page");
            }
        }
    }

    pub fn get_messages_to_persist(
        &self,
        result: &mut Vec<(SubPageId, Vec<Arc<MySbMessageContent>>)>,
    ) {
        match self {
            SubPage::SubPage(inner) => {
                if let Some(messages_to_persist) = inner.get_messages_to_persist() {
                    result.push((inner.sub_page_id, messages_to_persist));
                }
            }
            SubPage::AllMessagesMissing(_) => {}
        }
    }

    pub fn mark_messages_as_persisted(&mut self, ids: &QueueWithIntervals) {
        match self {
            SubPage::SubPage(inner) => inner.mark_messages_as_persisted(ids),
            SubPage::AllMessagesMissing(_) => {}
        }
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) {
        match self {
            SubPage::SubPage(inner) => inner.gc_messages(min_message_id),
            SubPage::AllMessagesMissing(_) => {}
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            SubPage::SubPage(inner) => inner.messages.is_empty(),
            SubPage::AllMessagesMissing(_) => true,
        }
    }

    pub fn get_min_message_to_persist(&self) -> Option<MessageId> {
        match self {
            SubPage::SubPage(inner) => inner.get_min_message_to_persist(),
            SubPage::AllMessagesMissing(_) => None,
        }
    }

    pub fn get_size_metrics(&self) -> SizeMetrics {
        match self {
            SubPage::SubPage(inner) => inner.get_size_metrics(),
            SubPage::AllMessagesMissing(_) => SizeMetrics {
                messages_amount: 0,
                data_size: 0,
                persist_size: 0,
                avg_message_size: 0,
            },
        }
    }

    pub fn is_ready_to_be_gc(&self, min_message_id: MessageId) -> bool {
        match self {
            SubPage::SubPage(inner) => inner.is_ready_to_gc(min_message_id),
            SubPage::AllMessagesMissing(inner) => {
                let min_message_sub_page_id: SubPageId = min_message_id.into();
                inner.sub_page_id.get_value() < min_message_sub_page_id.get_value()
            }
        }
    }
}

impl Into<SubPage> for SubPageInner {
    fn into(self) -> SubPage {
        SubPage::new(self)
    }
}
