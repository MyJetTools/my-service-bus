use std::collections::HashMap;

use my_service_bus_abstractions::publisher::MessageToPublish;
use my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::MySbMessageContent;

use my_service_bus_shared::page_id::PageId;
use my_service_bus_shared::sub_page::SubPageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use rust_extensions::lazy::LazyVec;

use crate::messages_page::MessagesPageList;
use crate::queue_subscribers::QueueSubscriber;
use crate::queues::{TopicQueue, TopicQueuesList};
use crate::sessions::SessionId;
use crate::utils::MinMessageIdCalculator;

use super::TopicMetrics;
const BADGE_HIGHLIGHT_TIME_OUT: u8 = 2;

pub struct TopicData {
    pub topic_id: String,
    pub message_id: MessageId,
    pub queues: TopicQueuesList,
    pub metrics: TopicMetrics,
    pub pages: MessagesPageList,
    pub publishers: HashMap<SessionId, u8>,
}

impl TopicData {
    pub fn new(topic_id: String, message_id: i64) -> Self {
        Self {
            topic_id,
            message_id: message_id.into(),
            queues: TopicQueuesList::new(),
            metrics: TopicMetrics::new(),
            pages: MessagesPageList::new(),
            publishers: HashMap::new(),
        }
    }

    #[inline]
    pub fn set_publisher_as_active(&mut self, session_id: SessionId) {
        self.publishers.insert(session_id, BADGE_HIGHLIGHT_TIME_OUT);
    }

    pub fn publish_messages(&mut self, session_id: SessionId, messages: Vec<MessageToPublish>) {
        self.set_publisher_as_active(session_id);

        let mut ids = QueueWithIntervals::new();

        for msg in messages {
            let message = MySbMessageContent {
                id: self.message_id.into(),
                content: msg.content,
                time: DateTimeAsMicroseconds::now(),
                headers: msg.headers,
            };

            ids.enqueue(message.id.into());

            let page_id = PageId::from_message_id(message.id);

            self.pages
                .get_or_create_page_mut(page_id)
                .publish_message(message);

            self.message_id.increment();
        }

        for topic_queue in self.queues.get_all_mut() {
            topic_queue.enqueue_messages(&ids);
        }
    }

    pub fn one_second_tick(&mut self) {
        for value in self.publishers.values_mut() {
            if *value > 0 {
                *value -= 1;
            }
        }

        self.queues.one_second_tick();
    }

    pub fn disconnect(
        &mut self,
        session_id: SessionId,
    ) -> Option<Vec<(&mut TopicQueue, QueueSubscriber)>> {
        self.publishers.remove(&session_id);

        self.queues.remove_subscribers_by_session_id(session_id)
    }

    pub fn get_min_message_id(&self) -> Option<MessageId> {
        let mut min_message_id = MinMessageIdCalculator::new();

        if self.message_id.get_value() > 1 {
            min_message_id.add(Some(self.message_id.get_value() - 1));
        }

        for topic_queue in self.queues.get_all() {
            let min_id = topic_queue.queue.get_min_id();
            min_message_id.add(min_id);
            min_message_id.add(topic_queue.subscribers.get_min_message_id());
            min_message_id.add(self.pages.get_persisted_min_message_id());
        }

        min_message_id.get()
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) {
        self.pages.gc_messages(min_message_id);
    }

    pub fn get_sub_pages_to_gc(
        &self,
        active_pages: &HashMap<i64, SubPageId>,
    ) -> Option<Vec<SubPageId>> {
        let mut result = LazyVec::new();

        for page in self.pages.get_pages() {
            for sub_page in page.sub_pages.values() {
                if !active_pages.contains_key(&sub_page.sub_page.sub_page_id.get_value()) {
                    if sub_page.messages_to_persist.len() == 0 {
                        result.add(sub_page.sub_page.sub_page_id);
                    }
                }
            }
        }

        result.get_result()
    }
}
