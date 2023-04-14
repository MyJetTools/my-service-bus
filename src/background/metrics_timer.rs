use std::sync::Arc;

use my_service_bus_abstractions::subscriber::TopicQueueType;
use rust_extensions::MyTimerTick;

use crate::app::AppContext;

pub struct MetricsTimer {
    app: Arc<AppContext>,
}

impl MetricsTimer {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for MetricsTimer {
    async fn tick(&self) {
        self.app.sessions.one_second_tick().await;

        let mut permanent_queues_without_subscribers = 0;
        let mut topics_without_queues = 0;

        for topic in self.app.topic_list.get_all().await {
            let mut topic_data = topic.get_access().await;
            topic_data.one_second_tick();

            let mut queues_count = 0;

            for queue in topic_data.queues.get_all() {
                let queue_size = queue.get_queue_size();
                queues_count += 1;
                self.app.prometheus.update_topic_queue_size(
                    topic.topic_id.as_str(),
                    queue.queue_id.as_str(),
                    queue_size,
                );

                let is_permanent = match &queue.queue_type {
                    TopicQueueType::Permanent => true,
                    TopicQueueType::DeleteOnDisconnect => false,
                    TopicQueueType::PermanentWithSingleConnection => true,
                };

                if is_permanent && queue.subscribers.get_amount() == 0 {
                    permanent_queues_without_subscribers += 1;
                }
            }

            if queues_count == 0 {
                topics_without_queues += 1;
            }

            let metrics = topic_data.pages.get_page_size_metrics();

            self.app
                .prometheus
                .update_topic_size_metrics(topic.topic_id.as_str(), &metrics);

            topic_data.metrics.one_second_tick(&metrics);
        }

        self.app
            .prometheus
            .update_permanent_queues_without_subscribers(permanent_queues_without_subscribers);

        self.app
            .prometheus
            .update_topics_without_queues(topics_without_queues);
    }
}
