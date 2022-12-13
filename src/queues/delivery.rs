use my_service_bus_shared::{
    sub_page::{SubPage, SubPageId},
    MySbMessageContent,
};
use rust_extensions::lazy::LazyVec;

use crate::messages_page::MessagesPageList;

use super::TopicQueue;

enum CompileSubpageResult<'s> {
    Ok(Vec<(i32, &'s MySbMessageContent)>),
    OkWithGcFound(Vec<(i32, &'s MySbMessageContent)>),
    GcFound,
    Empty,
}

impl TopicQueue {
    pub async fn try_to_deliver<'s>(
        &mut self,
        pages: &'s MessagesPageList,
        max_delivery_size: usize,
    ) -> Result<(), SubPageId> {
        if self.queue.len() == 0 {
            return Ok(());
        }

        while let Some(subscriber_id) = self.subscribers.find_subscriber_ready_to_deliver() {
            while let Some(message_id) = self.queue.peek() {
                let sub_page_id = SubPageId::from_message_id(message_id);

                let sub_page = pages.get_sub_page(sub_page_id);

                if sub_page.is_none() {
                    return Err(sub_page_id);
                }

                match self.compile_sub_page(sub_page_id, max_delivery_size, sub_page.unwrap()) {
                    CompileSubpageResult::Ok(to_publish) => {
                        self.subscribers
                            .get_by_id_mut(subscriber_id)
                            .unwrap()
                            .deliver_messages(to_publish)
                            .await;
                    }
                    CompileSubpageResult::OkWithGcFound(to_publish) => {
                        self.subscribers
                            .get_by_id_mut(subscriber_id)
                            .unwrap()
                            .deliver_messages(to_publish)
                            .await;
                        return Err(sub_page_id);
                    }
                    CompileSubpageResult::GcFound => {
                        return Err(sub_page_id);
                    }
                    CompileSubpageResult::Empty => {}
                }
            }
        }

        Ok(())
    }

    fn compile_sub_page<'s>(
        &mut self,
        sub_page_id: SubPageId,
        max_delivery_size: usize,
        sub_page: &'s SubPage,
    ) -> CompileSubpageResult<'s> {
        let mut data_size = 0;
        let mut result = LazyVec::new();
        let mut gced_found = false;
        while let Some(message_id) = self.queue.peek() {
            if data_size > max_delivery_size {
                break;
            }

            let msg_sub_page_id = SubPageId::from_message_id(message_id);

            if msg_sub_page_id.get_value() != sub_page_id.get_value() {
                break;
            };

            match sub_page.get_message(message_id) {
                my_service_bus_shared::sub_page::GetMessageResult::Message(msg) => {
                    self.queue.dequeue();
                    data_size += msg.content.len();
                    let attempt_no = self.delivery_attempts.get(msg.id);
                    result.add((attempt_no, msg));
                }
                my_service_bus_shared::sub_page::GetMessageResult::Missing => {
                    self.queue.dequeue();
                }
                my_service_bus_shared::sub_page::GetMessageResult::Gced => {
                    gced_found = true;
                    break;
                }
            }
        }

        let result = result.get_result();

        match result {
            Some(result) => {
                if gced_found {
                    CompileSubpageResult::OkWithGcFound(result)
                } else {
                    CompileSubpageResult::Ok(result)
                }
            }
            None => {
                if gced_found {
                    CompileSubpageResult::GcFound
                } else {
                    CompileSubpageResult::Empty
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use my_service_bus_abstractions::{publisher::MessageToPublish, subscriber::TopicQueueType};

    use crate::{
        sessions::{MyServiceBusSession, SessionConnection, TestConnectionData},
        topics::TopicData,
    };

    #[tokio::test]
    async fn test_generate_case_send_to_deliver() {
        const TOPIC_ID: &str = "test";
        const QUEUE_ID: &str = "test";

        let mut topic_data = TopicData::new(TOPIC_ID.to_string(), 0);

        let topic_queue = topic_data.queues.add_queue_if_not_exists(
            TOPIC_ID.to_string(),
            QUEUE_ID.to_string(),
            TopicQueueType::PermanentWithSingleConnection,
        );

        let session_id = 1;
        let connection = SessionConnection::Test(Arc::new(TestConnectionData::new(1, "127.0.0.1")));

        let session = MyServiceBusSession::new(session_id, connection);

        let session = Arc::new(session);

        topic_queue.subscribers.subscribe(
            1,
            TOPIC_ID.to_string(),
            QUEUE_ID.to_string(),
            session.clone(),
        );

        topic_data.publish_messages(vec![
            MessageToPublish {
                headers: None,
                content: vec![],
            },
            MessageToPublish {
                headers: None,
                content: vec![],
            },
        ]);

        let result = topic_data.try_to_deliver(1024).await;

        assert!(result.is_none());

        let result = session
            .connection
            .unwrap_as_test()
            .get_sent_messages_to_deliver()
            .await;

        assert_eq!(result.len(), 2);
    }
}
