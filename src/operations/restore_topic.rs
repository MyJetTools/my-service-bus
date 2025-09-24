use crate::app::AppContext;

pub async fn restore_topic(app: &AppContext, topic_id: &str) -> bool {
    if let Some(message_id) = app.persistence_client.restore_topic(topic_id).await {
        app.topic_list.restore(topic_id, message_id, true).await;
        true
    } else {
        false
    }
}
