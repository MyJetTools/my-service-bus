use crate::topics::TopicInner;

use super::delivery::SubscriberPackageBuilder;

#[cfg(not(test))]
pub fn send_new_messages_to_deliver(
    builder: SubscriberPackageBuilder,
    topic_data: &mut TopicInner,
    compilation_duration: std::time::Duration,
) {
    let subscriber_id = builder.subscriber_id;

    if let Some(queue) = topic_data.queues.get_mut(builder.queue_id.as_str()) {
        if let Some(subscriber) = queue.subscribers.get_by_id_mut(subscriber_id) {
            if builder.has_something_to_send() {
                subscriber.set_messages_on_delivery(
                    builder.messages_on_delivery.clone(),
                    compilation_duration,
                );

                subscriber.metrics.set_started_delivery();

                builder.send_messages_to_connection();
            } else {
                subscriber.cancel_the_rent();
            }
        }
    }
}

#[cfg(test)]
pub async fn send_new_messages_to_deliver(
    builder: SubscriberPackageBuilder,
    topic_data: &mut TopicInner,
    compilation_duration: std::time::Duration,
) {
    let subscriber_id = builder.subscriber_id;

    if let Some(queue) = topic_data.queues.get_mut(builder.queue_id.as_str()) {
        if let Some(subscriber) = queue.subscribers.get_by_id_mut(subscriber_id) {
            if builder.has_something_to_send() {
                subscriber.set_messages_on_delivery(
                    builder.messages_on_delivery.clone(),
                    compilation_duration,
                );

                subscriber.metrics.set_started_delivery();

                builder.send_messages_to_connection().await;
            } else {
                subscriber.cancel_the_rent();
            }
        }
    }
}
