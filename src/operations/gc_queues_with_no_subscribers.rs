use my_service_bus_abstractions::subscriber::TopicQueueType;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::AppContext, topics::TopicData};

pub fn gc_queues_with_no_subscribers(app: &AppContext, topic_data: &mut TopicData) {
    let now = DateTimeAsMicroseconds::now();

    let queues_with_no_subscribers = topic_data.queues.get_queues_with_no_subscribers();

    if queues_with_no_subscribers.is_none() {
        return;
    }

    let mut queues_to_delete = None;

    for topic_queue in queues_with_no_subscribers.unwrap() {
        if let TopicQueueType::DeleteOnDisconnect = topic_queue.queue_type {
            if now
                .duration_since(topic_queue.subscribers.last_unsubscribe)
                .as_positive_or_zero()
                > app.empty_queue_gc_timeout
            {
                println!("Detected DeleteOnDisconnect queue {}/{} with 0 subscribers. Last disconnect since {:?}", topic_data.topic_id, topic_queue.queue_id, topic_queue.subscribers.last_unsubscribe);

                if queues_to_delete.is_none() {
                    queues_to_delete = Some(Vec::new());
                }

                queues_to_delete
                    .as_mut()
                    .unwrap()
                    .push(topic_queue.queue_id.to_string());
            }
        }
    }

    if let Some(queues_to_delete) = queues_to_delete {
        for queue_id in queues_to_delete {
            topic_data.queues.remove(queue_id.as_str());
        }
    }
}
