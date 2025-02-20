use std::sync::Arc;

use my_logger::LogEventCtx;
use rust_extensions::MyTimerTick;
use tokio::sync::Mutex;

use crate::{app::AppContext, topics::ReusableTopicsList};

pub struct DeadSubscribersKickerTimer {
    app: Arc<AppContext>,
    reusable_topics_vec: Mutex<Option<ReusableTopicsList>>,
}

impl DeadSubscribersKickerTimer {
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
impl MyTimerTick for DeadSubscribersKickerTimer {
    async fn tick(&self) {
        let topics = self.get_reusable_topics_vec().await;

        for topic in topics.iter() {
            let dead_subscribers = topic
                .find_subscribers_dead_on_delivery(self.app.delivery_timeout)
                .await;

            if dead_subscribers.len() > 0 {
                for dead_subscriber in dead_subscribers {
                    my_logger::LOGGER.write_info(
                        "Dead subscribers detector".to_string(),
                        format!(
                            "Kicking Connection {} with dead subscriber {}",
                            dead_subscriber.session.get_session_id().get_value(),
                            dead_subscriber.subscriber_id.get_value()
                        ),
                        LogEventCtx::new()
                            .add("topicId", topic.topic_id.as_str())
                            .add("DeadTimeout", format!("{:?}", dead_subscriber.duration)),
                    );

                    dead_subscriber.session.disconnect().await;
                }
            }
        }

        self.put_reusable_topics_vec_back(topics).await;
    }
}
