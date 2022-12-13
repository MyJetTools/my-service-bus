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

            topic_data.gc_sub_pages();

            crate::operations::gc_queues_with_no_subscribers(self.app.as_ref(), &mut topic_data);

            topic_data.gc_messages();
        }

        crate::operations::gc_http_connections(self.app.as_ref()).await;
    }
}
