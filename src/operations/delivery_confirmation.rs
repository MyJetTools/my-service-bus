use std::sync::Arc;

use my_logger::LogEventCtx;
use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;

use crate::{app::AppContext, queue_subscribers::SubscriberId};

use super::OperationFailResult;

pub async fn all_confirmed(
    app: &Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    subscriber_id: SubscriberId,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    let mut topic_access = topic.get_access().await;

    let topic_queue =
        topic_access
            .queues
            .get_mut(queue_id)
            .ok_or(OperationFailResult::QueueNotFound {
                queue_id: queue_id.to_string(),
            })?;

    if let Err(err) = topic_queue.confirmed_delivered(subscriber_id) {
        my_logger::LOGGER.write_fatal_error(
            "confirm_delivery".to_string(),
            format!("{:?}", err),
            LogEventCtx::new()
                .add("topicId", topic.topic_id.as_str())
                .add("queueId", queue_id)
                .add("subscriberId", subscriber_id.get_value().to_string()),
        );
    }

    super::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_access);

    Ok(())
}

pub async fn all_fail(
    app: &Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    subscriber_id: SubscriberId,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    let mut topic_data = topic.get_access().await;

    {
        let topic_queue =
            topic_data
                .queues
                .get_mut(queue_id)
                .ok_or(OperationFailResult::QueueNotFound {
                    queue_id: queue_id.to_string(),
                })?;

        if let Err(err) = topic_queue.confirmed_non_delivered(subscriber_id) {
            my_logger::LOGGER.write_error(
                "confirm_non_delivery".to_string(),
                format!("{:?}", err),
                LogEventCtx::new()
                    .add("topicId", topic.topic_id.as_str())
                    .add("queueId", queue_id)
                    .add("subscriberId", subscriber_id.get_value().to_string()),
            );
        }
    }

    super::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data);

    Ok(())
}

pub async fn intermediary_confirm(
    app: &Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    subscriber_id: SubscriberId,
    confirmed: QueueWithIntervals,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    let mut topic_data = topic.get_access().await;

    {
        let topic_queue =
            topic_data
                .queues
                .get_mut(queue_id)
                .ok_or(OperationFailResult::QueueNotFound {
                    queue_id: queue_id.to_string(),
                })?;

        if let Err(err) = topic_queue.intermediary_confirmed(subscriber_id, confirmed) {
            my_logger::LOGGER.write_error(
                "some_messages_are_not_confirmed".to_string(),
                format!("{:?}", err),
                LogEventCtx::new()
                    .add("topicId", topic.topic_id.as_str())
                    .add("queueId", queue_id)
                    .add("subscriberId", subscriber_id.get_value().to_string()),
            );
        }
    }

    crate::operations::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data);

    Ok(())
}

pub async fn some_messages_are_confirmed(
    app: &Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    subscriber_id: SubscriberId,
    confirmed_messages: QueueWithIntervals,
) -> Result<(), OperationFailResult> {
    let topic = app
        .topic_list
        .get(topic_id)
        .await
        .ok_or(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        })?;

    let mut topic_data = topic.get_access().await;
    {
        let topic_queue =
            topic_data
                .queues
                .get_mut(queue_id)
                .ok_or(OperationFailResult::QueueNotFound {
                    queue_id: queue_id.to_string(),
                })?;

        if let Err(err) = topic_queue.confirmed_some_delivered(subscriber_id, confirmed_messages) {
            my_logger::LOGGER.write_fatal_error(
                "some_messages_are_confirmed".to_string(),
                format!("{:?}", err),
                LogEventCtx::new()
                    .add("topicId", topic.topic_id.as_str())
                    .add("queueId", queue_id)
                    .add("subscriberId", subscriber_id.get_value().to_string()),
            );
        }
    }

    super::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data);

    Ok(())
}
