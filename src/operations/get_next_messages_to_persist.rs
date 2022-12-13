use my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus_shared::{protobuf_models::MessageProtobufModel, sub_page::SubPageId};

use crate::topics::{Topic, TopicData};

pub async fn get_next_messages_to_persist(
    topic: &Topic,
) -> Option<(SubPageId, Vec<MessageProtobufModel>, QueueWithIntervals)> {
    let mut topic_data = topic.get_access().await;
    return get_messages_to_persist(&mut topic_data);
}

fn get_messages_to_persist(
    topic_data: &TopicData,
) -> Option<(SubPageId, Vec<MessageProtobufModel>, QueueWithIntervals)> {
    for sub_page in topic_data.pages.pages.values() {
        let mut messages_to_persist = Vec::with_capacity(sub_page.to_persist.len() as usize);

        let mut queue = QueueWithIntervals::new();

        for message_id in &sub_page.to_persist {
            if let Some(message) = sub_page.messages.get(&message_id) {
                let itm: MessageProtobufModel = message.into();
                messages_to_persist.push(itm);
                queue.enqueue(message_id);
            }
        }

        if messages_to_persist.len() > 0 {
            return Some((sub_page.sub_page_id, messages_to_persist, queue));
        }
    }

    return None;
}

#[cfg(test)]
mod tests {

    use my_service_bus_abstractions::publisher::MessageToPublish;

    use super::*;

    #[tokio::test]
    async fn test_no_messages_published() {
        const TOPIC_NAME: &str = "Test";
        let mut topic_data = TopicData::new(TOPIC_NAME.to_string(), 0);

        let result = get_messages_to_persist(&mut topic_data);

        assert_eq!(true, result.is_none());
    }

    #[test]
    fn test_some_messages_are_published() {
        const TOPIC_NAME: &str = "Test";

        let mut topic_data = TopicData::new(TOPIC_NAME.to_string(), 0);

        let msg = MessageToPublish {
            content: vec![0u8, 1u8, 2u8],
            headers: None,
        };

        topic_data.publish_messages(vec![msg]);

        let result = get_messages_to_persist(&mut topic_data);

        if let Some((_, messages, queue)) = result {
            assert_eq!(1, messages.len());
            assert_eq!(1, queue.len());
        } else {
            assert_eq!(true, result.is_none());
        }
    }
}
