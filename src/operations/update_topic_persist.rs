use crate::app::AppContext;

use super::OperationFailResult;

pub async fn update_topic_persist(
    app: &AppContext,
    topic_id: String,
    persist: bool,
) -> Result<(), OperationFailResult> {
    let topic = app.topic_list.get(topic_id.as_str()).await;

    if topic.is_none() {
        return Err(OperationFailResult::TopicNotFound { topic_id });
    }

    let topic = topic.unwrap();

    topic.update_persist(persist).await;

    Ok(())
}
