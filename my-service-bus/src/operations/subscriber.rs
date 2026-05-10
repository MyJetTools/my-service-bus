use std::sync::Arc;

use my_logger::LogEventCtx;
use my_service_bus::abstractions::subscriber::TopicQueueType;

use crate::{
    app::AppContext,
    queue_subscribers::{QueueSubscriber, SubscriberId},
    queues::TopicQueue,
    sessions::MyServiceBusSession,
};

use super::OperationFailResult;

pub async fn subscribe_to_queue(
    app: &Arc<AppContext>,
    topic_id: String,
    queue_id: String,
    queue_type: TopicQueueType,
    session: MyServiceBusSession,
) -> Result<SubscriberId, OperationFailResult> {
    let topic = {
        let topic = app.topic_list.get(topic_id.as_str());

        match topic {
            Some(result) => result,
            None => {
                if app.settings.auto_create_topic_on_subscribe {
                    app.topic_list.add_if_not_exists(topic_id.as_str())?
                } else {
                    return Err(OperationFailResult::TopicNotFound { topic_id });
                }
            }
        }
    };

    if topic.get_deleted() != 0 {
        return Err(OperationFailResult::TopicIsDeleted { topic_id });
    }

    let mut topic_data = topic.get_access();

    let topic_queue = topic_data.queues.add_queue_if_not_exists(
        topic.topic_id.clone(),
        queue_id,
        queue_type.clone(),
    );

    let subscriber_id = app.subscriber_id_generator.get_next_subscriber_id();

    topic_queue.update_queue_type(queue_type);

    let session_id = session.session_id;

    let kicked_subscribers = topic_queue.subscribers.subscribe(
        queue_type.is_single_connection(),
        subscriber_id,
        topic.topic_id.clone(),
        topic_queue.queue_id.clone(),
        session,
    );

    if kicked_subscribers.is_empty() {
        my_logger::LOGGER.write_info(
            "subscribe_to_queue",
            "Subscribed.",
            LogEventCtx::new()
                .add("topicId", topic_queue.queue_id.as_str())
                .add("queueId", topic_queue.queue_id.as_str())
                .add("subscriberId", subscriber_id.get_value().to_string())
                .add("sessionId", session_id.get_value().to_string()),
        );
    } else {
        for kicked_subscriber in kicked_subscribers {
            my_logger::LOGGER.write_info(
                "subscribe_to_queue",
                "Subscribed. Subscriber is kicked",
                LogEventCtx::new()
                    .add("topicId", topic_queue.queue_id.as_str())
                    .add("queueId", topic_queue.queue_id.as_str())
                    .add("subscriberId", subscriber_id.get_value().to_string())
                    .add(
                        "kickedSubscriberId",
                        kicked_subscriber.id.get_value().to_string(),
                    )
                    .add(
                        "messagesToDeliverOnKickSubscriber",
                        kicked_subscriber
                            .get_messages_amount_on_delivery()
                            .to_string(),
                    )
                    .add("sessionId", session_id.get_value().to_string()),
            );

            remove_subscriber(topic_queue, kicked_subscriber);
        }
    }

    crate::operations::delivery::try_to_deliver_to_subscribers(
        app.as_ref(),
        &topic,
        &mut topic_data,
    );

    Ok(subscriber_id)
}

pub fn remove_subscriber(queue: &mut TopicQueue, mut subscriber: QueueSubscriber) {
    let messages = subscriber.reset_delivery();

    if let Some(delivery_state_data) = &messages {
        queue.confirm_non_delivered(&delivery_state_data.bucket.to_be_confirmed);
    }
}

#[cfg(test)]
mod tests {
    use my_service_bus::abstractions::{
        publisher::MessageToPublish, subscriber::TopicQueueType, SbMessageHeaders,
    };

    #[tokio::test]
    async fn test_we_kick_subscriber_and_messages_goes_to_queue_back_and_then_to_new_connection() {
        const TOPIC_NAME: &str = "test-topic";
        const QUEUE_NAME: &str = "test-queue";
        let app = crate::test_tools::create_app_context().await;

        let session = app.sessions.add_test();

        let topic = crate::operations::create_topic_if_not_exists(
            &app,
            Some(session.session_id),
            TOPIC_NAME,
        )
        .await
        .unwrap();

        crate::operations::subscriber::subscribe_to_queue(
            &app,
            TOPIC_NAME.to_string(),
            QUEUE_NAME.to_string(),
            TopicQueueType::PermanentWithSingleConnection,
            session.clone().into(),
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

        let session2 = app.sessions.add_test();

        let subscriber_id_2 = crate::operations::subscriber::subscribe_to_queue(
            &app,
            TOPIC_NAME.to_string(),
            QUEUE_NAME.to_string(),
            TopicQueueType::PermanentWithSingleConnection,
            session2.clone().into(),
        )
        .await
        .unwrap();

        {
            let data = topic.get_access();
            let queue = data.queues.get(QUEUE_NAME).unwrap();

            let subscriber = queue.subscribers.get_by_id(subscriber_id_2).unwrap();

            assert_eq!(2, subscriber.get_messages_amount_on_delivery());
        }
    }
}
