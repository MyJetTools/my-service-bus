use std::sync::Arc;

use rust_extensions::MyTimerTick;

use crate::app::AppContext;

pub struct GcTimer {
    app: Arc<AppContext>,
}

impl GcTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for GcTimer {
    async fn tick(&self) {
        for topic in self.app.topic_list.get_all().await {
            let mut topic_data = topic.get_access().await;

            crate::operations::gc_message_pages(self.app.as_ref(), &mut topic_data);
            crate::operations::gc_queues_with_no_subscribers(self.app.as_ref(), &mut topic_data);

            if let Some(min_message_id) = topic_data.get_min_message_id() {
                topic_data.gc_messages(min_message_id);
            }
        }

        crate::operations::gc_http_connections(self.app.as_ref()).await;
    }
}
