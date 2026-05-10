use std::sync::Arc;

use crate::{app::AppContext, sessions::SessionId, topics::Topic};

use super::OperationFailResult;

pub async fn create_topic_if_not_exists(
    app: &Arc<AppContext>,
    session_id: Option<SessionId>,
    topic_id: &str,
) -> Result<Arc<Topic>, OperationFailResult> {
    let topic = app.topic_list.add_if_not_exists(topic_id)?;

    {
        if let Some(session_id) = session_id {
            let mut topic_data = topic.get_access();
            topic_data.set_publisher_as_active(session_id);
        }
    }

    return Ok(topic);
}
