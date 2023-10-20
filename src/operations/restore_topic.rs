use crate::app::AppContext;

pub async fn restore_topic(app: &AppContext, topic_id: &str) -> bool {
    app.messages_pages_repo.restore_topic(topic_id).await
}
