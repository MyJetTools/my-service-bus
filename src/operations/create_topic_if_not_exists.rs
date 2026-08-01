use std::sync::Arc;

use crate::{namespaces::Namespace, sessions::SessionId, topics::Topic};

use super::OperationFailResult;

pub async fn create_topic_if_not_exists(
    namespace: &Arc<Namespace>,
    session_id: Option<SessionId>,
    topic_id: &str,
) -> Result<Arc<Topic>, OperationFailResult> {
    let topic = namespace.topic_list.add_if_not_exists(topic_id)?;

    if topic.get_deleted() != 0 {
        return Err(OperationFailResult::TopicIsDeleted {
            topic_id: topic_id.to_string(),
        });
    }

    {
        if let Some(session_id) = session_id {
            let mut topic_data = topic.get_access();
            topic_data.set_publisher_as_active(session_id);
        }
    }

    return Ok(topic);
}
