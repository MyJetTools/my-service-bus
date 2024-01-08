use std::sync::Arc;

use my_http_server::HttpConnectionsCounter;
use my_tcp_sockets::ThreadsStatistics;
use rust_extensions::MyTimerTick;

use crate::app::AppContext;

pub struct MetricsTimer {
    app: Arc<AppContext>,
    http_connections_counter: HttpConnectionsCounter,
    threads_statistics: Arc<ThreadsStatistics>,
}

impl MetricsTimer {
    pub fn new(
        app: Arc<AppContext>,
        http_connections_counter: HttpConnectionsCounter,
        threads_statistics: Arc<ThreadsStatistics>,
    ) -> Self {
        Self {
            app,
            http_connections_counter,
            threads_statistics,
        }
    }
}

#[async_trait::async_trait]
impl MyTimerTick for MetricsTimer {
    async fn tick(&self) {
        self.app.sessions.one_second_tick().await;

        self.app
            .prometheus
            .update_tcp_threads(&self.threads_statistics);

        let mut permanent_queues_without_subscribers = 0;
        let mut topics_without_queues = 0;

        for topic in self.app.topic_list.get_all().await {
            let metrics = {
                let mut topic_data = topic.get_access().await;

                topic_data.one_second_tick();

                let mut queues_count = 0;

                for queue in topic_data.queues.get_all_mut() {
                    queue.one_second_tick();
                    let queue_size = queue.get_queue_size();
                    queues_count += 1;
                    self.app.prometheus.update_topic_queue_size(
                        topic.topic_id.as_str(),
                        queue.queue_id.as_str(),
                        queue_size,
                    );

                    if queue.is_permanent() && queue.subscribers.get_amount() == 0 {
                        permanent_queues_without_subscribers += 1;
                    }
                }

                if queues_count == 0 {
                    topics_without_queues += 1;
                }

                let metrics = topic_data.get_topic_size_metrics();

                topic_data.statistics.one_second_tick(&metrics);

                metrics
            };

            self.app
                .prometheus
                .update_topic_size_metrics(topic.topic_id.as_str(), &metrics);

            let http_connections_amount = self.http_connections_counter.get_connections_amount();

            self.app
                .prometheus
                .update_http_connections_amount(http_connections_amount);
        }

        self.app
            .prometheus
            .update_permanent_queues_without_subscribers(permanent_queues_without_subscribers);

        self.app
            .prometheus
            .update_topics_without_queues(topics_without_queues);
    }
}
