use std::sync::Arc;
use std::time::Duration;

use super::AppContext;

/// How long the shutdown keeps handing a topic's messages to the persistence. Without a limit a
/// persistence which is down holds the node forever: a stop never ends, and a Windows service is
/// killed by the Service Control Manager in the middle of it instead.
const PERSIST_ON_SHUTDOWN_LIMIT: Duration = Duration::from_secs(20);

pub async fn execute(app: Arc<AppContext>) {
    empty_persistence_queues(app.clone()).await;
    make_last_topics_and_queues_persist(app.clone()).await;
}

async fn empty_persistence_queues(app: Arc<AppContext>) {
    for namespace in app.namespaces.get_all().iter() {
        let topics = namespace.topic_list.get_all();

        'topics: for topic in topics.iter() {
            let started = std::time::Instant::now();

            // Read anew on every pass: the metrics are a copy, and a copy taken once never comes
            // down to zero - the loop went on for ever after the messages were persisted.
            loop {
                let persist_size = topic.get_topic_size_metrics().persist_size;

                if persist_size == 0 {
                    break;
                }

                if started.elapsed() > PERSIST_ON_SHUTDOWN_LIMIT {
                    println!(
                        "Topic {}/{} still has {} messages to persist after {:?}: the persistence does not take them. Shutting down without them",
                        namespace.name.as_str(),
                        topic.topic_id.as_str(),
                        persist_size,
                        PERSIST_ON_SHUTDOWN_LIMIT
                    );
                    continue 'topics;
                }

                println!(
                    "Topic {}/{} has {} messages to persist. Doing Force Persist",
                    namespace.name.as_str(),
                    topic.topic_id.as_str(),
                    persist_size
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
