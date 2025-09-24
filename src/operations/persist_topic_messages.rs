use std::sync::Arc;

use my_service_bus::shared::{protobuf_models::MessageProtobufModel, sub_page::SubPageId};

use crate::{app::AppContext, messages_page::MessagesToPersistBucket, topics::Topic};

//pub const PERSIST_PAYLOAD_MAX_SIZE: usize = 1024 * 1024 * 4;

pub async fn persist_topic_messages(app: &Arc<AppContext>, topic: &Arc<Topic>) {
    let messages_to_persist: Vec<(SubPageId, Vec<MessageProtobufModel>)> =
        topic.get_messages_to_persist(|itm| itm.into()).await;

    for (sub_page_id, messages_to_persist) in messages_to_persist {
        let mut bucket = MessagesToPersistBucket::new(sub_page_id);

        for msg in messages_to_persist {
            bucket.add(msg);
        }

        app.persistence_client
            .save_messages(topic.topic_id.as_str(), bucket.get())
            .await
            .unwrap();

        topic.mark_messages_as_persisted(&bucket).await;
    }
}
