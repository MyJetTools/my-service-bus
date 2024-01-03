use std::sync::Arc;

use my_logger::LogEventCtx;
use rust_extensions::MyTimerTick;

use crate::app::AppContext;

pub struct DeadSubscribersKickerTimer {
    app: Arc<AppContext>,
}

impl DeadSubscribersKickerTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for DeadSubscribersKickerTimer {
    async fn tick(&self) {
        let topics = self.app.topic_list.get_all().await;

        for topic in topics {
            if let Some(dead_subscribers) = topic
                .find_subscribers_dead_on_delivery(self.app.delivery_timeout)
                .await
            {
                for dead_subscriber in dead_subscribers {
                    my_logger::LOGGER.write_info(
                        "Dead subscribers detector".to_string(),
                        format!(
                            "Kicking Connection {} with dead subscriber {}",
                            dead_subscriber.session.id,
                            dead_subscriber.subscriber_id.get_value()
                        ),
                        LogEventCtx::new().add("topicId", topic.topic_id.as_str()),
                    );

                    dead_subscriber.session.disconnect().await;
                }
            }
        }
    }
}
