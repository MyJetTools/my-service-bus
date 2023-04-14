use std::sync::Arc;

use my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals;

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

    let mut topic_data = topic.get_access().await;

    let topic_queue =
        topic_data
            .queues
            .get_mut(queue_id)
            .ok_or(OperationFailResult::QueueNotFound {
                queue_id: queue_id.to_string(),
            })?;

    if let Err(err) = topic_queue.confirmed_delivered(subscriber_id) {
        app.logs.add_fatal_error(
            crate::app::logs::SystemProcess::DeliveryOperation,
            "confirm_delivery".to_string(),
            format!("{:?}", err),
            None,
        );
    }

    super::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data);

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
            app.logs.add_fatal_error(
                crate::app::logs::SystemProcess::DeliveryOperation,
                "confirm_non_delivery".to_string(),
                format!("{:?}", err),
                None,
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
            app.logs.add_fatal_error(
                crate::app::logs::SystemProcess::DeliveryOperation,
                "some_messages_are_not_confirmed".to_string(),
                format!("{:?}", err),
                None,
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
            app.logs.add_fatal_error(
                crate::app::logs::SystemProcess::DeliveryOperation,
                "some_messages_are_confirmed".to_string(),
                format!("{:?}", err),
                None,
            );
        }
    }

    super::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data);

    Ok(())
}
