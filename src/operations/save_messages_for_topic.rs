use std::sync::Arc;

use my_service_bus_shared::sub_page::SubPageId;

use crate::{app::AppContext, messages_page::MessagesToPersistBucket, topics::Topic};

pub async fn save_messages_for_topic(app: &Arc<AppContext>, topic: &Arc<Topic>) {
    while let Some((sub_page_id, mut messages_to_persist)) =
        super::get_next_messages_to_persist(topic.as_ref()).await
    {
        let messages = messages_to_persist.get();

        let result = app
            .messages_pages_repo
            .save_messages(topic.topic_id.as_str(), messages)
            .await;

        if let Err(err) = result {
            app.logs.add_error(
                Some(topic.topic_id.to_string()),
                crate::app::logs::SystemProcess::Timer,
                "persist_messages".to_string(),
                format!(
                    "Can not persist messages from id:{:?}",
                    messages_to_persist.first_message_id
                ),
                Some(format!("{:?}", err)),
            );
        } else {
            commit_persisted(topic.as_ref(), sub_page_id, &messages_to_persist).await;
        }
    }
}

async fn commit_persisted(
    topic: &Topic,
    sub_page_id: SubPageId,
    messages_to_persist: &MessagesToPersistBucket,
) {
    let mut topic_data = topic.get_access().await;
    topic_data.pages.commit_persisted_messages(
        topic.topic_id.as_str(),
        sub_page_id,
        messages_to_persist,
    );
}
