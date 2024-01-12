use std::sync::Arc;

use my_logger::LogEventCtx;

use crate::app::AppContext;

pub async fn persist_topics_and_queues(app: &Arc<AppContext>) {
    if let Some(get_persistence_version) = app.messages_pages_repo.get_persistence_version().await {
        app.persistence_version
            .update(get_persistence_version.as_str())
            .await;
    }

    let topics = app.topic_list.get_all().await;
    let mut topics_snapshots = Vec::with_capacity(topics.len());

    for topic in &topics {
        topics_snapshots.push(topic.get_topic_snapshot().await);
    }

    let result = app.topics_and_queues_repo.save(topics_snapshots).await;

    if let Err(err) = result {
        my_logger::LOGGER.write_error(
            "persist_topics_and_queues",
            format!("Failed to save topics and queues snapshot: {:?}", err),
            LogEventCtx::new(),
        );
    }

    for topic in &topics {
        crate::operations::persist_topic_messages(&app, topic).await;
    }
}
