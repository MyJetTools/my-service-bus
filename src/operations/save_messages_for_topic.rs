use std::sync::Arc;

use crate::{app::AppContext, topics::Topic};

pub const PERSIST_PAYLOAD_MAX_SIZE: usize = 1024 * 1024 * 4;

pub async fn save_messages_for_topic(app: &Arc<AppContext>, topic: &Arc<Topic>) {
    while let Some(mut messages_to_persist) = topic
        .get_messages_to_persist(PERSIST_PAYLOAD_MAX_SIZE)
        .await
    {
        let messages = messages_to_persist.get();

        if app.settings.persist_compressed {
            app.messages_pages_repo
                .save_messages(&topic.topic_id, messages)
                .await
                .unwrap();
        } else {
            app.messages_pages_repo
                .save_messages_uncompressed(&topic.topic_id, messages)
                .await
                .unwrap();
        }

        topic.mark_messages_as_persisted(&messages_to_persist).await;
    }
}
