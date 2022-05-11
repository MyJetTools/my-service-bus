use my_service_bus_shared::{
    messages_page::MessagesPagesCache,
    page_id::{get_page_id, PageId},
    queue_with_intervals::QueueWithIntervals,
    MySbMessageContent,
};

use crate::queue_subscribers::QueueSubscriber;

use super::delivery_attempts::DeliveryAttempts;

pub struct DeliveryIterator<'s> {
    pub topic_id: &'s str,
    pub queue_id: &'s str,
    pub pages: &'s MessagesPagesCache,
    pub queue: &'s mut QueueWithIntervals,
    pub delivery_attempts: &'s mut DeliveryAttempts,
    pub subscriber: &'s mut QueueSubscriber,
    yielded_size: usize,
    max_size: usize,
}

pub enum NextMessageResult<'s> {
    Value {
        content: &'s MySbMessageContent,
        attempt_no: i32,
    },
    LoadDataRequired(PageId),
}

impl<'s> DeliveryIterator<'s> {
    pub fn new(
        topic_id: &'s str,
        queue_id: &'s str,
        pages: &'s MessagesPagesCache,
        queue: &'s mut QueueWithIntervals,
        delivery_attempts: &'s mut DeliveryAttempts,
        subscriber: &'s mut QueueSubscriber,
        max_size: usize,
    ) -> Self {
        Self {
            topic_id,
            queue_id,
            pages,
            queue,
            delivery_attempts,
            max_size,
            subscriber,
            yielded_size: 0,
        }
    }
}

impl<'s> Iterator for DeliveryIterator<'s> {
    type Item = NextMessageResult<'s>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_message_id = self.queue.peek();

            if next_message_id.is_none() {
                return None;
            }

            let next_message_id = next_message_id.unwrap();

            let page_id = get_page_id(next_message_id);

            let result = self.pages.get_page(page_id);

            if result.is_none() {
                return Some(NextMessageResult::LoadDataRequired(page_id));
            }

            let message = result.unwrap().messages.get(&next_message_id);

            if message.is_none() {
                return Some(NextMessageResult::LoadDataRequired(page_id));
            }

            match message.unwrap() {
                my_service_bus_shared::MySbMessage::Loaded(content) => {
                    if self.yielded_size == 0
                        || self.yielded_size + content.content.len() <= self.max_size
                    {
                        self.queue.dequeue();
                        return Some(NextMessageResult::Value {
                            content,
                            attempt_no: self.delivery_attempts.get(next_message_id),
                        });
                    }

                    return None;
                }

                my_service_bus_shared::MySbMessage::Missing { id } => {
                    println!("Message with id {} is missing", id);
                    self.queue.dequeue();
                }
                my_service_bus_shared::MySbMessage::NotLoaded { id: _ } => {
                    return Some(NextMessageResult::LoadDataRequired(page_id))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use my_service_bus_tcp_shared::MessageToPublishTcpContract;

    use crate::{
        queues::delivery_iterator::NextMessageResult,
        sessions::{MyServiceBusSession, SessionConnection, SessionId, TestConnection},
        topics::TopicData,
    };

    #[test]
    pub fn test_on_delivery_no_queues() {
        let mut topic_data = TopicData::new("test".to_string(), 0);

        let msg = MessageToPublishTcpContract {
            headers: None,
            content: vec![0u8, 1u8, 2u8],
        };

        topic_data.publish_messages(1, vec![msg]);

        let result = topic_data.get_delivery_iterator(5 * 1024 * 1024);

        assert_eq!(true, result.is_none());
    }

    #[test]
    pub fn test_on_delivery_we_have_queue() {
        const SESSION_ID: SessionId = 13;

        let topic_id = "test";
        let queue_id = "test_queue";
        let mut topic_data = TopicData::new(topic_id.to_string(), 0);

        let session = Arc::new(MyServiceBusSession::new(
            SESSION_ID,
            SessionConnection::Test(TestConnection::new(15, "TestIp".to_string())),
        ));

        {
            let queue = topic_data.queues.add_queue_if_not_exists(
                topic_id.to_string(),
                queue_id.to_string(),
                my_service_bus_shared::queue::TopicQueueType::Permanent,
            );

            let prev_subscrber =
                queue
                    .subscribers
                    .subscribe(1, topic_id.to_string(), queue_id.to_string(), session);

            assert_eq!(prev_subscrber.is_none(), true);
        }

        let msg = MessageToPublishTcpContract {
            headers: None,
            content: vec![0u8, 1u8, 2u8],
        };

        topic_data.publish_messages(1, vec![msg]);

        let result = topic_data.get_delivery_iterator(5 * 1024 * 1024);

        assert_eq!(true, result.is_some());

        let result: Vec<NextMessageResult> = result.unwrap().collect();

        assert_eq!(1, result.len());

        match result[0] {
            NextMessageResult::Value {
                content,
                attempt_no,
            } => {
                assert_eq!(content.content, vec![0u8, 1u8, 2u8]);
                assert_eq!(attempt_no, 0);
            }
            NextMessageResult::LoadDataRequired(_) => {
                panic!("Should not be here")
            }
        }
    }
}
