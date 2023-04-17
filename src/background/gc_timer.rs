use std::{sync::Arc, time::Duration};

use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

use crate::app::AppContext;

const PAGE_GC_DELAY: Duration = Duration::from_secs(10);

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
            let now = DateTimeAsMicroseconds::now();
            let mut topic_data = topic.get_access().await;

            topic_data.gc_pages(now, PAGE_GC_DELAY);

            topic_data.gc_queues_with_no_subscribers(self.app.settings.queue_gc_timeout, now);

            if let Some(min_message_id) = topic_data.get_min_message_id() {
                topic_data
                    .pages
                    .gc_messages(min_message_id, now, PAGE_GC_DELAY);
            }
        }
        crate::operations::gc_http_connections(self.app.as_ref()).await;
    }
}
