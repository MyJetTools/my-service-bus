use std::sync::Arc;

use super::AppContext;

pub async fn execute(app: Arc<AppContext>) {
    empty_persistence_queues(app.clone()).await;
    make_last_topics_and_queues_persist(app.clone()).await;
}

async fn empty_persistence_queues(app: Arc<AppContext>) {
    for topic in app.topic_list.get_all().await {
        let metrics = topic.get_topic_size_metrics().await;

        while metrics.persist_size > 0 {
            println!(
                "Topic {} has {} messages to persist. Doing Force Persist",
                topic.topic_id, metrics.persist_size
            );

            crate::operations::save_messages_for_topic(&app, &topic).await;
        }

        println!("Topic {} has no messages to persist.", topic.topic_id);
    }
}

async fn make_last_topics_and_queues_persist(app: Arc<AppContext>) {
    println!("Making final topics and queues snapshot save");
    crate::operations::persist_topics_and_queues(&app).await;
    println!("Final topics and queues snapshot save is done");
}
