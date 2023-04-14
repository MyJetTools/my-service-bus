use std::sync::Arc;

use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

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
            let now = DateTimeAsMicroseconds::now();
            let mut topic_data = topic.get_access("GcTimer").await;

            println!("gc_message_pages {}", topic.topic_id);
            topic_data.gc_message_pages();

            println!("gc_queues_with_no_subscribers {}", topic.topic_id);
            topic_data.gc_queues_with_no_subscribers(self.app.settings.queue_gc_timeout, now);

            println!("get_min_message_id {}", topic.topic_id);
            //if let Some(min_message_id) = topic_data.get_min_message_id() {
            //    topic_data.gc_messages(min_message_id);
            // }
        }

        println!("GC get_min_message_id");
        crate::operations::gc_http_connections(self.app.as_ref()).await;

        println!("GC Done");
    }
}
