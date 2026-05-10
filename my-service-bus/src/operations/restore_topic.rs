use std::sync::Arc;

use crate::app::AppContext;

pub async fn restore_topic(app: &Arc<AppContext>, topic_id: &str) -> bool {
    let Some(topic) = app.topic_list.get(topic_id) else {
        return false;
    };

    if topic.get_deleted() == 0 {
        return false;
    }

    topic.set_deleted(0);

    let topic_list = app.topic_list.get_all();
    crate::operations::persist_topics_and_queues(app, topic_list.as_slice()).await;

    true
}
