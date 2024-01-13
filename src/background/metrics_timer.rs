use std::sync::Arc;

use my_http_server::HttpConnectionsCounter;
use my_tcp_sockets::ThreadsStatistics;
use rust_extensions::MyTimerTick;
use tokio::sync::Mutex;

use crate::{app::AppContext, topics::ReusableTopicsList};

pub struct MetricsTimer {
    app: Arc<AppContext>,
    http_connections_counter: HttpConnectionsCounter,
    threads_statistics: Arc<ThreadsStatistics>,
    reusable_topics_vec: Mutex<Option<ReusableTopicsList>>,
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
impl MyTimerTick for MetricsTimer {
    async fn tick(&self) {
        self.app.sessions.one_second_tick().await;

        self.app
            .prometheus
            .update_tcp_threads(&self.threads_statistics);

        let mut permanent_queues_without_subscribers = 0;
        let mut topics_without_queues = 0;

        let mut reusable_topics = self.get_reusable_topics_vec().await;

        self.app.topic_list.fill_topics(&mut reusable_topics).await;

        for topic in reusable_topics.iter() {
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

        self.put_reusable_topics_vec_back(reusable_topics).await;

        self.app
            .prometheus
            .update_permanent_queues_without_subscribers(permanent_queues_without_subscribers);

        self.app
            .prometheus
            .update_topics_without_queues(topics_without_queues);
    }
}
