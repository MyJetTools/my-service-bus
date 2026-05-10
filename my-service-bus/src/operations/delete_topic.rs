use std::sync::Arc;

use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::app::AppContext;

use super::OperationFailResult;

pub async fn delete_topic(
    app: &Arc<AppContext>,
    topic_id: &str,
    hard_delete_moment: DateTimeAsMicroseconds,
) -> Result<(), OperationFailResult> {
    let topic = app.topic_list.get(topic_id);

    if topic.is_none() {
        return Err(OperationFailResult::TopicNotFound {
            topic_id: topic_id.to_string(),
        });
    }

    app.persistence_client
        .delete_topic(topic_id, hard_delete_moment)
        .await;

    app.topic_list.delete_topic(topic_id);

    let topic_list = app.topic_list.get_all();
    crate::operations::persist_topics_and_queues(app, topic_list.as_slice()).await;

    Ok(())
}
