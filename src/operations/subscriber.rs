use std::sync::Arc;

use my_logger::LogEventCtx;
use my_service_bus::abstractions::subscriber::TopicQueueType;

use crate::{
    app::AppContext, queue_subscribers::QueueSubscriber, queues::TopicQueue,
    sessions::MyServiceBusSession,
};

use super::OperationFailResult;

pub async fn subscribe_to_queue(
    app: &Arc<AppContext>,
    topic_id: String,
    queue_id: String,
    queue_type: TopicQueueType,
    session: &Arc<MyServiceBusSession>,
) -> Result<(), OperationFailResult> {
    let topic = {
        let topic = app.topic_list.get(topic_id.as_str()).await;

        match topic {
            Some(result) => result,
            None => {
                if app.settings.auto_create_topic_on_subscribe {
                    app.topic_list.add_if_not_exists(topic_id.as_str()).await?
                } else {
                    return Err(OperationFailResult::TopicNotFound { topic_id });
                }
            }
        }
    };

    let mut topic_data = topic.get_access().await;

    let topic_queue = topic_data.queues.add_queue_if_not_exists(
        topic.topic_id.clone(),
        queue_id,
        queue_type.clone(),
    );

    let subscriber_id = app.subscriber_id_generator.get_next_subscriber_id();

    topic_queue.update_queue_type(queue_type);

    let kicked_subscriber_result = topic_queue.subscribers.subscribe(
        subscriber_id,
        topic.topic_id.to_string(),
        topic_queue.queue_id.clone(),
        session.clone(),
    );

    my_logger::LOGGER.write_info(
        "subscribe_to_queue",
        "Subscribed.",
        LogEventCtx::new()
            .add("topicId", topic_queue.queue_id.as_str())
            .add("queueId", topic_queue.queue_id.as_str())
            .add("subscriberId", subscriber_id.get_value().to_string())
            .add("sessionId", session.id.get_value().to_string()),
    );

    if let Some(kicked_subscriber) = kicked_subscriber_result {
        remove_subscriber(topic_queue, kicked_subscriber);
    }

    super::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data);

    Ok(())
}

pub fn remove_subscriber(queue: &mut TopicQueue, mut subscriber: QueueSubscriber) {
    let messages = subscriber.reset_delivery();

    if let Some(delivery_bucket) = &messages {
        queue.confirm_non_delivered(&delivery_bucket.ids);
    }
}
