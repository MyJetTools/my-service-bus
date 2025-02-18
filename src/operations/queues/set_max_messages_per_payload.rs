use super::super::OperationFailResult;

use crate::app::AppContext;
pub async fn set_max_messages_per_payload(
    app: &AppContext,
    topic_id: &str,
    queue_id: &str,
    max_amount: Option<usize>,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    let mut topic_data = topic.get_access().await;

    let topic_queue =
        topic_data
            .queues
            .get_mut(queue_id)
            .ok_or(OperationFailResult::QueueNotFound {
                queue_id: queue_id.to_string(),
            })?;

    topic_queue.max_messages_per_payload = max_amount;

    Ok(())
}
