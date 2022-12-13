use std::sync::Arc;

use my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus_shared::sub_page::SubPageId;

use crate::{app::AppContext, topics::Topic};

pub async fn save_messages_for_topic(app: &Arc<AppContext>, topic: &Arc<Topic>) {
    while let Some((sub_page_id, messages_to_persist, queue)) =
        super::get_next_messages_to_persist(topic.as_ref()).await
    {
        let result = if app.persist_compressed {
            app.messages_pages_repo
                .save_messages(topic.topic_id.as_str(), messages_to_persist)
                .await
        } else {
            app.messages_pages_repo
                .save_messages_uncompressed(topic.topic_id.as_str(), messages_to_persist)
                .await
        };

        if let Err(err) = result {
            commit_persisted(topic.as_ref(), sub_page_id, queue.clone()).await;

            crate::LOGS.add_error(
                Some(topic.topic_id.to_string()),
                crate::app::logs::SystemProcess::Timer,
                "persist_messages".to_string(),
                format!(
                    "Can not persist messages from id:{:?}. Err: {:?}",
                    queue.peek(),
                    err
                ),
                None,
            );
        } else {
            commit_persisted(topic.as_ref(), sub_page_id, queue).await;
        }
    }
}

async fn commit_persisted(topic: &Topic, sub_page_id: SubPageId, persisted: QueueWithIntervals) {
    let mut topic_data = topic.get_access().await;
    topic_data
        .pages
        .commit_persisted_messages(sub_page_id, persisted);
}
