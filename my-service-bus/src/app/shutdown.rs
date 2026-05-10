use std::sync::Arc;

use super::AppContext;

pub async fn execute(app: Arc<AppContext>) {
    empty_persistence_queues(app.clone()).await;
    make_last_topics_and_queues_persist(app.clone()).await;
}

async fn empty_persistence_queues(app: Arc<AppContext>) {
    let topics = app.topic_list.get_all();
    for topic in topics.iter() {
        let metrics = topic.get_topic_size_metrics();

        while metrics.persist_size > 0 {
            println!(
                "Topic {} has {} messages to persist. Doing Force Persist",
                topic.topic_id.as_str(),
                metrics.persist_size
            );

            crate::operations::persist_topic_messages(&app, &topic).await;
        }

        println!(
            "Topic {} has no messages to persist.",
            topic.topic_id.as_str()
        );
    }
}

async fn make_last_topics_and_queues_persist(app: Arc<AppContext>) {
    println!("Making final topics and queues snapshot save");

    let topic_list = app.topic_list.get_all();

    crate::operations::persist_topics_and_queues(&app, topic_list.as_slice()).await;
    println!("Final topics and queues snapshot save is done");
}
