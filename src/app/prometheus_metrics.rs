use prometheus::{Encoder, IntGauge, IntGaugeVec, Opts, Registry, TextEncoder};

use crate::topics::TopicData;

pub struct PrometheusMetrics {
    registry: Registry,
    pub persist_queue_size: IntGaugeVec,
    pub topic_queue_size: IntGaugeVec,
    permanent_queues_without_subscribers: IntGauge,
    topics_without_queues: IntGauge,
    topic_data_size: IntGaugeVec,
    topic_messages_amount: IntGaugeVec,
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

        return Self {
            registry,
            persist_queue_size,
            topic_queue_size,
            permanent_queues_without_subscribers,
            topics_without_queues,
            topic_data_size,
            topic_messages_amount,
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

    pub fn update_topic_size_metrics(&self, topic_data: &TopicData) {
        let metrics = topic_data.pages.get_metrics();

        self.topic_data_size
            .with_label_values(&[&topic_data.topic_id])
            .set(metrics.content_size as i64);

        self.persist_queue_size
            .with_label_values(&[&topic_data.topic_id])
            .set(metrics.to_persist_size as i64);

        self.topic_messages_amount
            .with_label_values(&[&topic_data.topic_id])
            .set(metrics.loaded_messages_amount as i64);
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
}

fn create_topic_persist_queue_size() -> IntGaugeVec {
    let gauge_opts = Opts::new("topic_persist_queue_size", "Topic queue to persist size");

    let lables = &["topic"];

    IntGaugeVec::new(gauge_opts, lables).unwrap()
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

fn create_topic_messages_amount() -> IntGaugeVec {
    let gauge_opts = Opts::new("topic_messages_amount", "Messages amount in cache");

    let lables = &["topic"];

    IntGaugeVec::new(gauge_opts, lables).unwrap()
}

fn create_topics_without_queues() -> IntGauge {
    IntGauge::new("topics_without_queues", "Topics without queues").unwrap()
}
