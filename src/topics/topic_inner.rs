use std::sync::Arc;
use std::time::Duration;

use my_service_bus::abstractions::publisher::MessageToPublish;
use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus::abstractions::MessageId;

use my_service_bus::shared::sub_page::SubPageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::avg_value::AvgValue;
use crate::messages_page::{ActiveSubPages, MessagesPageList, MySbMessageContent, SizeMetrics};
use crate::queue_subscribers::QueueSubscriber;
use crate::queues::{TopicQueue, TopicQueuesList};
use crate::sessions::SessionId;
use crate::utils::MinMessageIdCalculator;

use super::{TopicPublishers, TopicStatistics};

const BADGE_HIGHLIGHT_TIME_OUT: u8 = 2;

pub struct TopicInner {
    pub topic_id: String,
    pub message_id: MessageId,
    pub queues: TopicQueuesList,
    pub statistics: TopicStatistics,
    pub pages: MessagesPageList,
    pub publishers: TopicPublishers,
    pub persist: bool,
    pub avg_size: AvgValue,
}

impl TopicInner {
    pub fn new(topic_id: String, message_id: i64, persist: bool) -> Self {
        Self {
            topic_id,
            message_id: message_id.into(),
            queues: TopicQueuesList::new(),
            statistics: TopicStatistics::new(),
            pages: MessagesPageList::new(),
            publishers: TopicPublishers::new(),
            persist,
            avg_size: AvgValue::new(),
        }
    }

    #[inline]
    pub fn set_publisher_as_active(&mut self, session_id: SessionId) {
        self.publishers.add(session_id, BADGE_HIGHLIGHT_TIME_OUT);
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

            self.avg_size.add(message.content.len());

            ids.enqueue(message.id.into());

            let page_id: SubPageId = message.id.into();

            let page = self.pages.get_or_create_mut(page_id);
            page.update_last_accessed(message.time);
            page.add_message(message, self.persist);

            self.message_id.increment();
        }

        for topic_queue in self.queues.get_all_mut() {
            topic_queue.enqueue_messages(&ids);
        }
    }

    pub fn one_second_tick(&mut self) {
        self.publishers.one_second_tick();
    }

    pub fn disconnect(
        &mut self,
        session_id: SessionId,
    ) -> Option<Vec<(&mut TopicQueue, QueueSubscriber)>> {
        self.publishers.remove(session_id);

        self.queues.remove_subscribers_by_session_id(session_id)
    }

    pub fn get_min_message_id(&self) -> Option<MessageId> {
        let mut min_message_id = MinMessageIdCalculator::new();

        min_message_id.add(Some(self.message_id.get_value()));

        min_message_id.add(self.pages.get_persisted_min_message_id());

        for topic_queue in self.queues.get_all() {
            let min_id = topic_queue.queue.get_min_id();
            min_message_id.add(min_id);
            min_message_id.add(topic_queue.subscribers.get_min_message_id());
        }

        min_message_id.get()
    }

    pub fn get_active_sub_pages(&self) -> ActiveSubPages {
        let mut result = ActiveSubPages::new();

        let sub_page_id: SubPageId = self.message_id.into();

        result.add_if_not_exists(sub_page_id);

        if let Some(message_id) = self.pages.get_persisted_min_message_id() {
            let sub_page_id: SubPageId = message_id.into();
            result.add_if_not_exists(sub_page_id);
        }

        for queue in self.queues.get_all() {
            if let Some(min_msg_id) = queue.get_min_msg_id() {
                let sub_page_id = SubPageId::from_message_id(min_msg_id);

                result.add_if_not_exists(sub_page_id);
            }
        }

        result
    }

    pub fn get_current_sub_page(&self) -> SubPageId {
        let sub_page_id: SubPageId = self.message_id.into();

        sub_page_id
    }

    pub fn gc_pages(&mut self) {
        if let Some(min_message_id) = self.get_min_message_id() {
            let active_sub_pages = self.get_active_sub_pages();
            self.pages.gc_pages(&active_sub_pages, min_message_id);
        }
    }

    pub fn gc_queues_with_no_subscribers(
        &mut self,
        queue_gc_timeout: Duration,
        now: DateTimeAsMicroseconds,
    ) -> Option<Vec<String>> {
        let queues_with_no_subscribers = self.queues.get_queues_with_no_subscribers();

        if queues_with_no_subscribers.is_none() {
            return None;
        }

        let mut queues_to_delete = None;

        for topic_queue in queues_with_no_subscribers.unwrap() {
            if let my_service_bus::abstractions::subscriber::TopicQueueType::DeleteOnDisconnect =
                topic_queue.queue_type
            {
                if now
                    .duration_since(topic_queue.subscribers.last_unsubscribe)
                    .as_positive_or_zero()
                    > queue_gc_timeout
                {
                    println!("Detected DeleteOnDisconnect queue {}/{} with 0 subscribers. Last disconnect since {:?}", self.topic_id, topic_queue.queue_id, topic_queue.subscribers.last_unsubscribe);

                    if queues_to_delete.is_none() {
                        queues_to_delete = Some(Vec::new());
                    }

                    queues_to_delete
                        .as_mut()
                        .unwrap()
                        .push(topic_queue.queue_id.to_string());
                }
            }
        }

        if let Some(queues_to_delete) = &queues_to_delete {
            for queue_id in queues_to_delete {
                self.queues.remove(queue_id.as_str());
            }
        }

        queues_to_delete
    }

    pub fn get_topic_size_metrics(&self) -> SizeMetrics {
        let mut result = SizeMetrics::new(self.avg_size.get());

        for sub_page in self.pages.sub_pages.values() {
            let metrics = sub_page.get_size_metrics();
            result.append_without_avg(&metrics);
        }

        result
    }

    pub fn gc_messages(&mut self) {
        if let Some(min_message_id) = self.get_min_message_id() {
            let current_sub_page_id = self.get_current_sub_page();
            self.pages.gc_messages(min_message_id, current_sub_page_id);
        }
    }

    pub fn get_messages_to_persist(&self) -> Vec<(SubPageId, Vec<Arc<MySbMessageContent>>)> {
        let mut result = Vec::with_capacity(2);
        self.pages.get_messages_to_persist(&mut result);
        result
    }

    pub fn mark_messages_as_persisted(&mut self, sub_page_id: SubPageId, ids: &QueueWithIntervals) {
        self.pages.mark_messages_as_persisted(sub_page_id, ids);
        self.gc_messages();
    }

    #[cfg(test)]
    pub fn get_message<'s>(
        &'s self,
        message_id: MessageId,
    ) -> Option<crate::messages_page::GetMessageResult<'s>> {
        let sub_page_id: SubPageId = message_id.into();
        let sub_page = self.pages.get(sub_page_id)?;

        Some(sub_page.get_message(message_id))
    }
}

#[cfg(test)]
mod tests {
    use my_service_bus::abstractions::{
        publisher::{MessageToPublish, SbMessageHeaders},
        queue_with_intervals::QueueWithIntervals,
        subscriber::TopicQueueType,
    };

    #[test]
    fn test_we_deliver_then_persist_then_gc_message() {
        let mut topic_inner = super::TopicInner::new("test".to_string(), 0, true);

        topic_inner.queues.add_queue_if_not_exists(
            "test".to_string(),
            "test".to_string(),
            TopicQueueType::DeleteOnDisconnect,
        );

        topic_inner.publish_messages(
            10.into(),
            vec![MessageToPublish {
                headers: SbMessageHeaders::new(),
                content: vec![1, 2, 3],
            }],
        );

        let queue = topic_inner.queues.get_mut("test").unwrap();

        let message_to_deliver_id = queue.queue.dequeue().unwrap();

        let mut delivered = QueueWithIntervals::new();

        delivered.enqueue(message_to_deliver_id);

        queue.confirm_delivered(&delivered);

        let messages_to_persist = topic_inner.get_messages_to_persist();

        for (sub_page_id, messages) in messages_to_persist {
            let mut confirm_persisted = QueueWithIntervals::new();
            for msg in messages {
                confirm_persisted.enqueue(msg.id.get_value());
            }

            topic_inner.mark_messages_as_persisted(sub_page_id, &confirm_persisted);
        }

        let message_result = topic_inner.get_message(message_to_deliver_id.into());

        assert!(message_result.unwrap().is_garbage_collected());
    }
}
