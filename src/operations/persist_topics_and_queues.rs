use std::sync::Arc;

use my_logger::LogEventCtx;

use crate::{app::AppContext, topics::ReusableTopicsList};

pub async fn persist_topics_and_queues(
    app: &Arc<AppContext>,
    reusable_topics: &mut ReusableTopicsList,
) {
    if let Some(get_persistence_version) = app.messages_pages_repo.get_persistence_version().await {
        app.persistence_version
            .update(get_persistence_version.as_str())
            .await;
    }

    app.topic_list.fill_topics(reusable_topics).await;

    let mut topics_snapshots = Vec::with_capacity(reusable_topics.len());

    for topic in reusable_topics.iter() {
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

    for topic in reusable_topics.iter() {
        crate::operations::persist_topic_messages(&app, topic).await;
    }
}
