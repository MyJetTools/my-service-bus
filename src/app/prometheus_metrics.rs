use my_tcp_sockets::ThreadsStatistics;
use prometheus::{Encoder, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder};

use crate::messages_page::SizeMetrics;

const TCP_METRIC: &str = "tcp_metric";

pub struct PrometheusMetrics {
    registry: Registry,
    pub persist_queue_size: IntGaugeVec,
    pub topic_queue_size: IntGaugeVec,
    permanent_queues_without_subscribers: IntGauge,
    topics_without_queues: IntGauge,
    topic_data_size: IntGaugeVec,
    topic_mean_message_size: IntGaugeVec,
    topic_messages_amount: IntGaugeVec,
    http_connections_amount: IntGauge,
    tcp_connections: IntGaugeVec,
}

impl PrometheusMetrics {
    pub fn new() -> Self {
        let registry = Registry::new();

        let persist_queue_size = create_topic_persist_queue_size();

        let permanent_queues_without_subscribers = create_permanent_queues_without_subscribers();

        let topic_queue_size = create_topic_queue_size();

        let topics_without_queues = create_topics_without_queues();

        let topic_data_size = create_topic_data_size();

        let topic_messages_amount = create_topic_messages_amount();

        let http_connections_amount = create_http_connections_amount();

        let topic_mean_message_size = create_topic_mean_message_size();

        let tcp_connections = create_tcp_connections();

        registry
            .register(Box::new(tcp_connections.clone()))
            .unwrap();

        registry
            .register(Box::new(http_connections_amount.clone()))
            .unwrap();

        registry
            .register(Box::new(topic_queue_size.clone()))
            .unwrap();

        registry
            .register(Box::new(persist_queue_size.clone()))
            .unwrap();

        registry
            .register(Box::new(permanent_queues_without_subscribers.clone()))
            .unwrap();

        registry
            .register(Box::new(topics_without_queues.clone()))
            .unwrap();

        registry
            .register(Box::new(topic_data_size.clone()))
            .unwrap();

        registry
            .register(Box::new(topic_messages_amount.clone()))
            .unwrap();

        registry
            .register(Box::new(topic_mean_message_size.clone()))
            .unwrap();

        return Self {
            registry,
            persist_queue_size,
            topic_queue_size,
            permanent_queues_without_subscribers,
            topics_without_queues,
            topic_data_size,
            topic_messages_amount,
            http_connections_amount,
            topic_mean_message_size,
            tcp_connections,
        };
    }

    pub fn update_topic_queue_size(&self, topic_id: &str, queue_id: &str, value: i64) {
        self.topic_queue_size
            .with_label_values(&[topic_id, queue_id])
            .set(value);
    }

    pub fn update_permanent_queues_without_subscribers(&self, value: i64) {
        self.permanent_queues_without_subscribers.set(value);
    }

    pub fn update_topics_without_queues(&self, value: i64) {
        self.topics_without_queues.set(value);
    }

    pub fn update_topic_size_metrics(&self, topic_id: &str, metrics: &SizeMetrics) {
        self.topic_data_size
            .with_label_values(&[topic_id])
            .set(metrics.data_size as i64);

        self.persist_queue_size
            .with_label_values(&[topic_id])
            .set(metrics.persist_size as i64);

        self.topic_messages_amount
            .with_label_values(&[topic_id])
            .set(metrics.messages_amount as i64);

        self.topic_mean_message_size
            .with_label_values(&[topic_id])
            .set(metrics.avg_message_size as i64);
    }

    pub fn build(&self) -> Vec<u8> {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        buffer
    }

    pub fn queue_is_deleted(&self, topic_id: &str, queue_id: &str) {
        let result = self
            .topic_queue_size
            .remove_label_values(&[topic_id, queue_id]);

        println!(
            "Error during removing topic_queue_size from metrics for Topic:{}, Queue:{}: {:?}",
            topic_id, queue_id, result
        );
    }

    pub fn update_http_connections_amount(&self, amount: i64) {
        self.http_connections_amount.set(amount);
    }

    pub fn mark_new_tcp_connection(&self) {
        self.tcp_connections.with_label_values(&["count"]).inc();
    }

    pub fn mark_new_tcp_disconnection(&self) {
        self.tcp_connections.with_label_values(&["count"]).dec();
    }

    pub fn update_tcp_threads(&self, threads_statistics: &ThreadsStatistics) {
        self.tcp_connections
            .with_label_values(&["ping_threads"])
            .set(threads_statistics.get_ping_threads() as i64);

        self.tcp_connections
            .with_label_values(&["read_threads"])
            .set(threads_statistics.get_read_threads() as i64);

        self.tcp_connections
            .with_label_values(&["write_threads"])
            .set(threads_statistics.get_write_threads() as i64);
    }
}

fn create_topic_persist_queue_size() -> IntGaugeVec {
    let gauge_opts = Opts::new("topic_persist_queue_size", "Topic queue to persist size");

    let labels = &["topic"];

    IntGaugeVec::new(gauge_opts, labels).unwrap()
}

fn create_topic_queue_size() -> IntGaugeVec {
    let gauge_opts = Opts::new("topic_queue_size", "Topic queue size");

    let lables = &["topic", "queue"];

    // This unwraps runs on application start. If it fails - applications is not started
    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_permanent_queues_without_subscribers() -> IntGauge {
    IntGauge::new(
        "permanent_queues_without_subscribers",
        "Permanent queues without subscribers count",
    )
    .unwrap()
}

fn create_topic_data_size() -> IntGaugeVec {
    let gauge_opts = Opts::new("topic_data_size", "Topic data size");

    let lables = &["topic"];

    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_topic_mean_message_size() -> IntGaugeVec {
    let gauge_opts = Opts::new("topic_mean_message_size", "Topic mean message size");

    let lables = &["topic"];

    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_topic_messages_amount() -> IntGaugeVec {
    let gauge_opts = Opts::new("topic_messages_amount", "Messages amount in cache");

    let lables = &["topic"];

    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_topics_without_queues() -> IntGauge {
    IntGauge::new("topics_without_queues", "Topics without queues").unwrap()
}

fn create_http_connections_amount() -> IntGauge {
    IntGauge::new("http_connections_amount", "Amount of Http Connections").unwrap()
}

fn create_tcp_connections() -> IntGaugeVec {
    let gauge_opts = Opts::new(format!("tcp_connections"), format!("Tcp Connections"));
    let labels = &[TCP_METRIC];
    IntGaugeVec::new(gauge_opts, labels).unwrap()
}
