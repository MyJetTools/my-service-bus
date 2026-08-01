use std::sync::Arc;

use super::AppContext;

pub async fn execute(app: Arc<AppContext>) {
    empty_persistence_queues(app.clone()).await;
    make_last_topics_and_queues_persist(app.clone()).await;
}

async fn empty_persistence_queues(app: Arc<AppContext>) {
    for namespace in app.namespaces.get_all().iter() {
        let topics = namespace.topic_list.get_all();

        for topic in topics.iter() {
            let metrics = topic.get_topic_size_metrics();

            while metrics.persist_size > 0 {
                println!(
                    "Topic {}/{} has {} messages to persist. Doing Force Persist",
                    namespace.name.as_str(),
                    topic.topic_id.as_str(),
                    metrics.persist_size
                );

                crate::operations::persist_topic_messages(&app, &topic).await;
            }

            println!(
                "Topic {}/{} has no messages to persist.",
                namespace.name.as_str(),
                topic.topic_id.as_str()
            );
        }
    }
}

async fn make_last_topics_and_queues_persist(app: Arc<AppContext>) {
    println!("Making final topics and queues snapshot save");

    crate::operations::persist_all(&app).await;
    println!("Final topics and queues snapshot save is done");
}
