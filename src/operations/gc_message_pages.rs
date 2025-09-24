#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use my_service_bus::{
        abstractions::{publisher::MessageToPublish, subscriber::TopicQueueType, SbMessageHeaders},
        shared::sub_page::SubPageId,
    };

    use crate::{grpc_client::PersistenceGrpcService, settings::SettingsModel};

    #[tokio::test]
    async fn test_that_we_do_not_gc_messages_which_are_on_delivery() {
        const TOPIC_NAME: &str = "test-topic";
        const QUEUE_NAME: &str = "test-queue";
        const DELIVERY_SIZE: usize = 16;

        let settings = SettingsModel::create_test_settings(DELIVERY_SIZE);

        let app = Arc::new(
            crate::app::AppContext::new(
                PersistenceGrpcService::create_mock_instance(),
                settings.into(),
            )
            .await,
        );

        let session = app.sessions.add_test("127.0.0.1").await;

        let topic = crate::operations::publisher::create_topic_if_not_exists(
            &app,
            Some(session.session_id),
            TOPIC_NAME,
        )
        .await
        .unwrap();

        let subscriber_id = crate::operations::subscriber::subscribe_to_queue(
            &app,
            TOPIC_NAME.to_string(),
            QUEUE_NAME.to_string(),
            TopicQueueType::PermanentWithSingleConnection,
            session.clone(),
        )
        .await
        .unwrap();

        let msg1 = MessageToPublish {
            headers: SbMessageHeaders::new(),
            content: vec![0u8, 1u8, 2u8],
        };

        let msg2 = MessageToPublish {
            headers: SbMessageHeaders::new(),
            content: vec![3u8, 4u8, 5u8],
        };

        crate::operations::publisher::publish(
            &app,
            TOPIC_NAME,
            vec![msg1, msg2],
            false,
            session.session_id,
        )
        .await
        .unwrap();

        {
            let mut data = topic.get_access().await;

            data.gc_messages();
            data.gc_pages();

            let queue = data.queues.get(QUEUE_NAME).unwrap();

            let subscriber = queue.subscribers.get_by_id(subscriber_id).unwrap();

            assert_eq!(2, subscriber.get_messages_amount_on_delivery());

            let sub_page = data.pages.get(SubPageId::new(0)).unwrap();

            let messages = sub_page.unwrap_all_messages_with_content();

            assert_eq!(messages.len(), 2);
        }
    }
}
