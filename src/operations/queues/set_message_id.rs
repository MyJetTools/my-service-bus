use my_service_bus::abstractions::MessageId;

use crate::app::AppContext;

use super::super::OperationFailResult;

pub async fn set_message_id(
    app: &AppContext,
    topic_id: &str,
    queue_id: &str,
    message_id: MessageId,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    let mut topic_data = topic.get_access().await;

    let topic_message_id = topic_data.message_id;

    let topic_queue =
        topic_data
            .queues
            .get_mut(queue_id)
            .ok_or(OperationFailResult::QueueNotFound {
                queue_id: queue_id.to_string(),
            })?;

    topic_queue.set_message_id(message_id, topic_message_id);

    Ok(())
}

pub async fn delete_queue(
    app: &AppContext,
    topic_id: &str,
    queue_id: &str,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    let mut topic_data = topic.get_access().await;

    topic_data.queues.remove(queue_id);

    app.prometheus.queue_is_deleted(topic_id, queue_id);

    Ok(())
}
