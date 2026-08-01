use std::sync::Arc;

use crate::{app::AppContext, namespaces::Namespace};

pub async fn restore_topic(
    app: &Arc<AppContext>,
    namespace: &Arc<Namespace>,
    topic_id: &str,
) -> bool {
    let Some(topic) = namespace.topic_list.get(topic_id) else {
        return false;
    };

    if topic.get_deleted() == 0 {
        return false;
    }

    topic.set_deleted(0);

    crate::operations::persist_all(app).await;

    true
}
