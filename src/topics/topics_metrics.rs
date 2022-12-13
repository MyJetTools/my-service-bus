use crate::metric_data::{MetricOneSecond, MetricsHistory};

pub struct TopicMetrics {
    messages_per_second_going: MetricOneSecond,
    packets_per_second_going: MetricOneSecond,

    pub messages_per_second: usize,
    pub packets_per_second: usize,

    pub publish_history: MetricsHistory,
}

impl TopicMetrics {
    pub fn new() -> Self {
        Self {
            messages_per_second_going: MetricOneSecond::new(),
            packets_per_second_going: MetricOneSecond::new(),
            messages_per_second: 0,
            packets_per_second: 0,
            publish_history: MetricsHistory::new(),
        }
    }

    pub fn update_topic_metrics(&mut self, new_messages_count: usize) {
        self.messages_per_second_going.increase(new_messages_count);

        self.packets_per_second_going.increase(1);
    }

    pub fn one_second_tick(&mut self) {
        // todo!("Commented out because of ....????'")
        // self.size_metrics.update(metrics);

        let messages_per_second = self.messages_per_second_going.get_and_reset();
        self.packets_per_second = self.packets_per_second_going.get_and_reset();
        self.messages_per_second = messages_per_second;

        self.publish_history.put(messages_per_second as i32);
    }
}
