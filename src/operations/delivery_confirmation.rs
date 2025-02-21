use std::sync::Arc;

use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;

use crate::{
    app::AppContext,
    queue_subscribers::SubscriberId,
    queues::{DeliveryBucket, TopicQueue},
};

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

    {
        let topic_queue =
            topic_access
                .queues
                .get_mut(queue_id)
                .ok_or(OperationFailResult::QueueNotFound {
                    queue_id: queue_id.to_string(),
                })?;

        if let Some(delivery_bucket) = get_delivery_bucket(topic_queue, subscriber_id, true) {
            topic_queue.confirm_delivered(&delivery_bucket.to_be_confirmed);
        }

        if !topic_access.persist {
            topic_access.gc_messages();
        }
    }

    #[cfg(test)]
    crate::operations::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_access)
        .await;
    #[cfg(not(test))]
    crate::operations::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_access);

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

        if let Some(delivery_bucket) = get_delivery_bucket(topic_queue, subscriber_id, false) {
            topic_queue.confirm_non_delivered(&delivery_bucket.to_be_confirmed);
        }
    }

    #[cfg(test)]
    crate::operations::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data).await;
    #[cfg(not(test))]
    crate::operations::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data);

    Ok(())
}

pub async fn intermediary_confirm(
    app: &Arc<AppContext>,
    topic_id: &str,
    queue_id: &str,
    subscriber_id: SubscriberId,
    confirmed_ids: QueueWithIntervals,
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

        {
            let subscriber = topic_queue.subscribers.get_by_id_mut(subscriber_id);

            if let Some(subscriber) = subscriber {
                if let Some(result) = subscriber.intermediary_confirmed(&confirmed_ids) {
                    if result.confirmed_amount > 0 {
                        subscriber.metrics.set_delivered_statistic(
                            result.confirmed_amount,
                            result.confirm_duration,
                        );
                    }
                }
            }
        }

        topic_queue.confirm_delivered(&confirmed_ids);
    }

    #[cfg(test)]
    crate::operations::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data).await;
    #[cfg(not(test))]
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

        if let Some(mut delivery_bucket) = get_delivery_bucket(topic_queue, subscriber_id, false) {
            delivery_bucket.confirmed(&confirmed_messages);
            topic_queue.confirm_non_delivered(&delivery_bucket.to_be_confirmed);
        }
    }

    #[cfg(test)]
    crate::operations::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data).await;
    #[cfg(not(test))]
    crate::operations::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data);

    Ok(())
}

fn get_delivery_bucket(
    topic_queue: &mut TopicQueue,
    subscriber_id: SubscriberId,
    positive: bool,
) -> Option<DeliveryBucket> {
    let subscriber = topic_queue.subscribers.get_by_id_mut(subscriber_id);

    if subscriber.is_none() {
        println!(
            "{}/{} Can not find subscriber {} to confirm '{}' delivery",
            topic_queue.topic_id.as_str(),
            topic_queue.queue_id.as_str(),
            subscriber_id.get_value(),
            if positive { "positive" } else { "negative" }
        );

        return None;
    }

    let subscriber = subscriber.unwrap();

    let mut delivery_bucket = subscriber.reset_delivery();

    if let Some(delivery_bucket) = &mut delivery_bucket {
        let delivery_amount = delivery_bucket.to_be_confirmed.queue_size();
        if delivery_amount > 0 {
            subscriber.update_delivery_time(delivery_amount, positive);
        } else {
            println!(
                "{}/{} No messages on delivery at subscriber {}",
                topic_queue.topic_id.as_str(),
                topic_queue.queue_id.as_str(),
                subscriber_id.get_value()
            );
            return None;
        }
    } else {
        if delivery_bucket.is_none() {
            println!(
                "{}/{}: No messages basket on delivery at subscriber {}",
                topic_queue.topic_id.as_str(),
                topic_queue.queue_id.as_str(),
                subscriber_id.get_value()
            );
        };
    }

    delivery_bucket
}
