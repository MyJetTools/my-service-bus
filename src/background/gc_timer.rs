use std::sync::Arc;

use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

use crate::app::AppContext;

//const PAGE_GC_DELAY: Duration = Duration::from_secs(10);

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

            let removed_queues = {
                let mut topic_data = topic.get_access().await;
                topic_data.gc_messages();
                topic_data.gc_pages();

                let removed_queues = topic_data
                    .gc_queues_with_no_subscribers(self.app.settings.queue_gc_timeout, now);

                removed_queues
            };

            if let Some(removed_queues) = &removed_queues {
                for queue_id in removed_queues {
                    self.app
                        .prometheus
                        .queue_is_deleted(&topic.topic_id, queue_id);
                }
            }
        }
        crate::operations::gc_http_connections(self.app.as_ref()).await;
    }
}
