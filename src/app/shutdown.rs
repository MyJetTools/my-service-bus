use std::sync::Arc;

use super::AppContext;

pub async fn execute(app: Arc<AppContext>) {
    empty_persistence_queues(app.clone()).await;
    make_last_topcis_and_queues_persist(app.clone()).await;
}

async fn empty_persistence_queues(app: Arc<AppContext>) {
    for topic in app.topic_list.get_all().await {
        let amount_to_persist = {
            let topic_data = topic.get_access().await;
            topic_data.pages.get_amount_to_persist()
        };

        while amount_to_persist > 0 {
            println!(
                "Topic {} has {} messages to persist. Doing Force Persist",
                topic.topic_id, amount_to_persist
            );

            crate::operations::save_messages_for_topic(&app, &topic).await;
        }

        println!("Topic {} has no messages to persist.", topic.topic_id);
    }
}

async fn make_last_topcis_and_queues_persist(app: Arc<AppContext>) {
    println!("Making final topics and queues snapshot save");
    crate::operations::persist_topics_and_queues(&app).await;
    println!("Final topics and queues snapshot save is done");
}
