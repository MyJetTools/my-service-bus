use std::sync::Arc;

use my_logger::LogEventCtx;
use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick};

use crate::app::AppContext;

pub struct GcDeletedTopicsTimer {
    app: Arc<AppContext>,
}

impl GcDeletedTopicsTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for GcDeletedTopicsTimer {
    async fn tick(&self) {
        let now = DateTimeAsMicroseconds::now().unix_microseconds;

        let topics = self.app.topic_list.get_all();

        for topic in topics.iter() {
            let deleted = topic.get_deleted();
            if deleted == 0 || deleted > now {
                continue;
            }

            let topic_id = topic.topic_id.as_str();

            match self.app.persistence_client.hard_delete_topic(topic_id).await {
                Ok(()) => {
                    self.app.topic_list.delete_topic(topic_id);

                    my_logger::LOGGER.write_info(
                        "GcDeletedTopics",
                        format!("Topic {} hard-deleted", topic_id),
                        LogEventCtx::new().add("topicId", topic_id),
                    );
                }
                Err(err) => {
                    my_logger::LOGGER.write_error(
                        "GcDeletedTopics",
                        format!(
                            "Failed to hard-delete topic {}. Will retry next tick. Err: {:?}",
                            topic_id, err
                        ),
                        LogEventCtx::new().add("topicId", topic_id),
                    );
                }
            }
        }
    }
}
