use std::sync::Arc;

use my_service_bus_abstractions::subscriber::TopicQueueType;

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
    let mut topic = app.topic_list.get(topic_id.as_str()).await;

    if topic.is_none() {
        if app.settings.auto_create_topic_on_subscribe {
            topic = Some(app.topic_list.add_if_not_exists(topic_id.as_str()).await?);
        } else {
            return Err(OperationFailResult::TopicNotFound { topic_id });
        }
    }

    let topic = topic.unwrap();

    let mut topic_data = topic.get_access().await;

    let topic_queue = topic_data.queues.add_queue_if_not_exists(
        topic.topic_id.to_string(),
        queue_id,
        queue_type.clone(),
    );

    let subscriber_id = app.subscriber_id_generator.get_next_subsriber_id();

    topic_queue.update_queue_type(queue_type);

    let kicked_subscriber_result = topic_queue.subscribers.subscribe(
        subscriber_id,
        topic.topic_id.to_string(),
        topic_queue.queue_id.to_string(),
        session.clone(),
    );

    app.logs.add_info(
        Some(topic.topic_id.to_string()),
        crate::app::logs::SystemProcess::QueueOperation,
        format!(
            "Subscribed. SessionId: {}. SubscriberId: {}",
            session.id, subscriber_id
        ),
        format!(
            "Session {} is subscribing to the {}/{} ",
            session.id, topic.topic_id, topic_queue.queue_id
        ),
        None,
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
        queue.mark_not_delivered(delivery_bucket);
    }
}
