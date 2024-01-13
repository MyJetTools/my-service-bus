use std::sync::Arc;

use futures_util::lock::Mutex;
use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

use crate::{app::AppContext, topics::ReusableTopicsList};

//const PAGE_GC_DELAY: Duration = Duration::from_secs(10);

pub struct GcTimer {
    app: Arc<AppContext>,
    reusable_topics_vec: Mutex<Option<ReusableTopicsList>>,
}

impl GcTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self {
            app,
            reusable_topics_vec: Mutex::new(None),
        }
    }

    async fn get_reusable_topics_vec(&self) -> ReusableTopicsList {
        let mut result = self.reusable_topics_vec.lock().await;

        match result.take() {
            Some(topics) => topics,
            None => ReusableTopicsList::new(),
        }
    }

    async fn put_reusable_topics_vec_back(&self, topics: ReusableTopicsList) {
        let mut result = self.reusable_topics_vec.lock().await;
        *result = Some(topics);
    }
}

#[async_trait::async_trait]
impl MyTimerTick for GcTimer {
    async fn tick(&self) {
        let mut topic_list = self.get_reusable_topics_vec().await;

        self.app.topic_list.fill_topics(&mut topic_list).await;

        for topic in topic_list.iter() {
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

        self.put_reusable_topics_vec_back(topic_list).await;

        crate::operations::gc_http_connections(self.app.as_ref()).await;
    }
}
