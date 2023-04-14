use my_service_bus_shared::sub_page::SubPageId;

use crate::{
    messages_page::MessagesToPersistBucket,
    topics::{Topic, TopicData},
};

pub async fn get_next_messages_to_persist(
    topic: &Topic,
) -> Option<(SubPageId, MessagesToPersistBucket)> {
    let mut topic_data = topic.get_access("get_next_messages_to_persist").await;
    return get_messages_to_persist(&mut topic_data);
}

fn get_messages_to_persist(
    topic_data: &mut TopicData,
) -> Option<(SubPageId, MessagesToPersistBucket)> {
    for page in topic_data.pages.get_pages_mut() {
        if let Some(sub_page_data) = page.get_sub_page_with_messages_to_persist() {
            let messages_to_persist =
                sub_page_data.compile_messages_to_persist(topic_data.topic_id.as_str());
            return Some((sub_page_data.sub_page.sub_page_id, messages_to_persist));
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

        topic_data.publish_messages(1, vec![msg]);

        let result = get_messages_to_persist(&mut topic_data);

        if let Some((_, mut messages)) = result {
            assert_eq!(1, messages.get().len());
        } else {
            assert_eq!(true, result.is_none());
        }
    }
}
