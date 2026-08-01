use std::sync::Arc;

use my_logger::LogEventCtx;
use rust_extensions::{MyTimerTick, RepeatTimerIteration};

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
    async fn tick(&self) -> RepeatTimerIteration {
        for namespace in self.app.namespaces.get_all().iter() {
            for topic in namespace.topic_list.get_all().iter() {
                let dead_subscribers =
                    topic.find_subscribers_dead_on_delivery(self.app.delivery_timeout);

                for dead_subscriber in dead_subscribers {
                    my_logger::LOGGER.write_info(
                        "Dead subscribers detector".to_string(),
                        format!(
                            "Kicking Connection {} with dead subscriber {}",
                            dead_subscriber.session.session_id.get_value(),
                            dead_subscriber.subscriber_id.get_value()
                        ),
                        LogEventCtx::new()
                            .add("namespace", namespace.name.as_str())
                            .add("topicId", topic.topic_id.as_str())
                            .add("DeadTimeout", format!("{:?}", dead_subscriber.duration)),
                    );

                    dead_subscriber.session.disconnect().await;
                }
            }
        }

        RepeatTimerIteration::WithInterval
    }
}
