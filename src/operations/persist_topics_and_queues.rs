use std::sync::Arc;

use crate::app::AppContext;

pub async fn persist_topics_and_queues(app: &Arc<AppContext>) {
    if let Some(get_persistence_version) = app.messages_pages_repo.get_persistence_version().await {
        let mut write_access = app.persistence_version.lock().await;
        *write_access = get_persistence_version;
    }

    let topics = app.topic_list.get_all().await;
    let mut topics_snapshots = Vec::new();

    for topic in &topics {
        topics_snapshots.push(topic.get_topic_snapshot().await);
    }

    let result = app.topics_and_queues_repo.save(topics_snapshots).await;

    if let Err(err) = result {
        app.logs.add_error(
            None,
            crate::app::logs::SystemProcess::TcpSocket,
            "persist::sync_topics_and_queues".to_string(),
            "Failed to save topics and queues snapshot".to_string(),
            Some(format!("{:?}", err)),
        );
    }

    for topic in &topics {
        crate::operations::save_messages_for_topic(&app, topic).await;
    }
}
