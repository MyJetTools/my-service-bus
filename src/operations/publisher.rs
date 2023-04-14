use std::sync::Arc;

use my_service_bus_abstractions::publisher::MessageToPublish;

use crate::{app::AppContext, sessions::SessionId, topics::Topic};

use super::OperationFailResult;

pub async fn create_topic_if_not_exists(
    app: &Arc<AppContext>,
    session_id: Option<SessionId>,
    topic_id: &str,
) -> Result<Arc<Topic>, OperationFailResult> {
    let topic = app.topic_list.add_if_not_exists(topic_id).await?;

    crate::operations::persist_topics_and_queues(&app).await;

    {
        if let Some(session_id) = session_id {
            let mut topic_data = topic.get_access("create_topic_if_not_exists").await;
            topic_data.set_publisher_as_active(session_id);
        }
    }

    return Ok(topic);
}

pub async fn publish(
    app: &Arc<AppContext>,
    topic_id: &str,
    messages: Vec<MessageToPublish>,
    persist_immediately: bool,
    session_id: SessionId,
) -> Result<(), OperationFailResult> {
    if app.states.is_shutting_down() {
        return Err(OperationFailResult::ShuttingDown);
    }

    let topic = app.topic_list.get(topic_id).await;

    if topic.is_none() {
        if app.settings.auto_create_topic_on_publish {
            app.topic_list.add_if_not_exists(topic_id).await?;
        } else {
            return Err(OperationFailResult::TopicNotFound {
                topic_id: topic_id.to_string(),
            });
        }
    }

    let topic = topic.unwrap();

    let mut topic_data = topic.get_access("publish").await;

    let messages_count = messages.len();

    topic_data.publish_messages(session_id, messages);

    topic_data.metrics.update_topic_metrics(messages_count);

    if persist_immediately {
        let prev = topic
            .immediately_persist_is_charged
            .swap(true, std::sync::atomic::Ordering::SeqCst);

        if !prev {
            app.immediately_persist_event_loop.send(topic.clone());
        }
    }

    super::delivery::try_to_deliver_to_subscribers(&app, &topic, &mut topic_data);
    Ok(())
}
