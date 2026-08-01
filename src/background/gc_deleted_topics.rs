use std::sync::Arc;

use my_logger::LogEventCtx;
use rust_extensions::{date_time::DateTimeAsMicroseconds, MyTimerTick, RepeatTimerIteration};

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
    async fn tick(&self) -> RepeatTimerIteration {
        let now = DateTimeAsMicroseconds::now().unix_microseconds;

        for namespace in self.app.namespaces.get_all().iter() {
            for topic in namespace.topic_list.get_all().iter() {
                let deleted = topic.get_deleted();
                if deleted == 0 || deleted > now {
                    continue;
                }

                let topic_id = topic.topic_id.as_str();

                match self
                    .app
                    .persistence_client
                    .hard_delete_topic(namespace.as_grpc_namespace(), topic_id)
                    .await
                {
                    Ok(()) => {
                        namespace.topic_list.delete_topic(topic_id);

                        my_logger::LOGGER.write_info(
                            "GcDeletedTopics",
                            format!("Topic {}/{} hard-deleted", namespace.name, topic_id),
                            LogEventCtx::new()
                                .add("namespace", namespace.name.as_str())
                                .add("topicId", topic_id),
                        );
                    }
                    Err(err) => {
                        my_logger::LOGGER.write_error(
                            "GcDeletedTopics",
                            format!(
                                "Failed to hard-delete topic {}/{}. Will retry next tick. Err: {:?}",
                                namespace.name, topic_id, err
                            ),
                            LogEventCtx::new()
                                .add("namespace", namespace.name.as_str())
                                .add("topicId", topic_id),
                        );
                    }
                }
            }
        }

        RepeatTimerIteration::WithInterval
    }
}
