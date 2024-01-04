use std::time::Duration;

use super::{
    GetMessageResult, MessagesToPersistBucket, MissingSubPageInner, MySbMessageContent,
    SizeMetrics, SubPageInner,
};
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

    pub fn get_messages_to_persist(&self, max_size: usize) -> Option<MessagesToPersistBucket> {
        match self {
            SubPage::SubPage(inner) => inner.get_messages_to_persist(max_size),
            SubPage::AllMessagesMissing(_) => None,
        }
    }

    pub fn mark_messages_as_persisted(&mut self, ids: &QueueWithIntervals) {
        match self {
            SubPage::SubPage(inner) => inner.mark_messages_as_persisted(ids),
            SubPage::AllMessagesMissing(_) => {}
        }
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) -> bool {
        match self {
            SubPage::SubPage(inner) => inner.gc_messages(min_message_id),
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
            },
        }
    }
    pub fn ready_to_be_gc(&self, now: DateTimeAsMicroseconds, gc_delay: Duration) -> bool {
        let last_access_time = match self {
            SubPage::SubPage(inner) => {
                if inner.has_messages_to_persist() {
                    return false;
                }

                inner.last_accessed.as_date_time()
            }
            SubPage::AllMessagesMissing(inner) => inner.last_accessed.as_date_time(),
        };

        now.duration_since(last_access_time).as_positive_or_zero() > gc_delay
    }
}

impl Into<SubPage> for SubPageInner {
    fn into(self) -> SubPage {
        SubPage::new(self)
    }
}
