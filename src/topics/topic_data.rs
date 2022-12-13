use std::collections::HashMap;

use my_service_bus_abstractions::publisher::MessageToPublish;
use my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::sub_page::SubPageId;
use my_service_bus_shared::MySbMessageContent;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use rust_extensions::lazy::LazyVec;

use crate::messages_page::MessagesPageList;
use crate::queue_subscribers::QueueSubscriber;
use crate::queues::{TopicQueue, TopicQueuesList};
use crate::sessions::SessionId;
use crate::utils::MinMessageIdCalculator;

use super::TopicMetrics;
const BADGE_HIGHLIGHT_TIMOUT: u8 = 2;

pub struct TopicData {
    pub topic_id: String,
    pub message_id: MessageId,
    pub queues: TopicQueuesList,
    pub metrics: TopicMetrics,
    pub pages: MessagesPageList,
    pub publishers: HashMap<SessionId, u8>,
}

impl TopicData {
    pub fn new(topic_id: String, message_id: MessageId) -> Self {
        Self {
            topic_id,
            message_id,
            queues: TopicQueuesList::new(),
            metrics: TopicMetrics::new(),
            pages: MessagesPageList::new(),
            publishers: HashMap::new(),
        }
    }

    #[inline]
    pub fn set_publisher_as_active(&mut self, session_id: SessionId) {
        self.publishers.insert(session_id, BADGE_HIGHLIGHT_TIMOUT);
    }

    pub fn publish_messages(&mut self, messages: Vec<MessageToPublish>) {
        let mut ids = QueueWithIntervals::new();

        for msg in messages {
            let message = MySbMessageContent {
                id: self.message_id,
                content: msg.content,
                time: DateTimeAsMicroseconds::now(),
                headers: msg.headers,
            };

            ids.enqueue(message.id);

            self.pages.publish_brand_new_message(message);

            self.message_id = self.message_id + 1;
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

    pub fn get_active_sub_pages(&self) -> HashMap<SubPageId, ()> {
        let mut result = HashMap::new();

        result.insert(SubPageId::from_message_id(self.message_id), ());

        for queue in self.queues.get_queues() {
            if let Some(min_message_id) = queue.get_min_msg_id() {
                result.insert(SubPageId::from_message_id(min_message_id), ());
            }
        }

        result
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

        if self.message_id > 0 {
            min_message_id.add(Some(self.message_id - 1));
        }

        for topic_queue in self.queues.get_all() {
            let min_id = topic_queue.queue.get_min_id();
            min_message_id.add(min_id);
            min_message_id.add(topic_queue.subscribers.get_min_message_id());
            min_message_id.add(self.pages.get_persisted_min_message_id());
        }

        min_message_id.value
    }

    pub fn gc_messages(&mut self) {
        let min_msg_id = self.get_min_message_id();

        if min_msg_id.is_none() {
            return;
        }

        let min_msg_id = min_msg_id.unwrap();

        self.pages.gc_messages(min_msg_id);
    }

    pub fn gc_sub_pages(&mut self) {
        let active_pages = self.get_active_sub_pages();
        self.pages.gc_sub_pages(&active_pages);
    }

    pub async fn try_to_deliver(&mut self, max_packet_size: usize) -> Option<Vec<SubPageId>> {
        let mut result = LazyVec::new();
        for topic_queue in self.queues.get_all_mut() {
            if let Err(sub_page_id) = topic_queue
                .try_to_deliver(&mut self.pages, max_packet_size)
                .await
            {
                result.add(sub_page_id);
            }
        }

        result.get_result()
    }
}

#[cfg(test)]
mod tests {
    use my_service_bus_abstractions::{
        queue_with_intervals::{QueueIndexRange, QueueWithIntervals},
        subscriber::TopicQueueType,
    };
    use my_service_bus_shared::sub_page::{SubPageId, SUB_PAGE_MESSAGES_AMOUNT};

    use super::TopicData;

    #[test]
    fn test_get_active_pages_on_brand_new_topic() {
        let topic_data = TopicData::new("test".to_string(), 0);

        let active_pages = topic_data.get_active_sub_pages();

        assert_eq!(1, active_pages.len());
        assert_eq!(true, active_pages.contains_key(&SubPageId::new(0)))
    }
    #[test]
    fn test_get_active_pages_when_one_of_queue_on_previous_page() {
        let mut topic_data = TopicData::new("test".to_string(), 0);

        topic_data.message_id = SUB_PAGE_MESSAGES_AMOUNT * 1;

        topic_data.queues.restore(
            "test".to_string(),
            "test-queue".to_string(),
            TopicQueueType::Permanent,
            QueueWithIntervals {
                intervals: vec![QueueIndexRange {
                    from_id: 0,
                    to_id: topic_data.message_id - 1,
                }],
            },
        );

        let active_pages = topic_data.get_active_sub_pages();

        assert_eq!(2, active_pages.len());
        assert_eq!(true, active_pages.contains_key(&SubPageId::new(0)));
        assert_eq!(true, active_pages.contains_key(&SubPageId::new(1)));
    }
}

//todo!("Write test for gc_sub_pages");
