use std::sync::Arc;

use crate::namespaces::Namespace;

use super::OperationFailResult;

pub async fn update_topic_persist(
    namespace: &Arc<Namespace>,
    topic_id: String,
    persist: bool,
) -> Result<(), OperationFailResult> {
    let topic = namespace.topic_list.get(topic_id.as_str());

    if topic.is_none() {
        return Err(OperationFailResult::TopicNotFound { topic_id });
    }

    let topic = topic.unwrap();

    topic.update_persist(persist);

    Ok(())
}
