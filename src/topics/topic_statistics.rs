use crate::{
    messages_page::SizeMetrics,
    metric_data::{MetricOneSecond, MetricsHistory},
};

pub struct TopicStatistics {
    messages_per_second_going: MetricOneSecond,
    packets_per_second_going: MetricOneSecond,

    pub messages_per_second: usize,
    pub packets_per_second: usize,

    pub publish_history: MetricsHistory,

    pub size_metrics: SizeMetrics,
}

impl TopicStatistics {
    pub fn new() -> Self {
        Self {
            messages_per_second_going: MetricOneSecond::new(),
            packets_per_second_going: MetricOneSecond::new(),
            messages_per_second: 0,
            packets_per_second: 0,
            publish_history: MetricsHistory::new(),

            size_metrics: SizeMetrics::new(),
        }
    }

    pub fn update_messages_count(&mut self, new_messages_count: usize) {
        self.messages_per_second_going.increase(new_messages_count);

        self.packets_per_second_going.increase(1);
    }

    pub fn one_second_tick(&mut self, metrics: &SizeMetrics) {
        self.size_metrics.update(metrics);

        let messages_per_second = self.messages_per_second_going.get_and_reset();
        self.packets_per_second = self.packets_per_second_going.get_and_reset();
        self.messages_per_second = messages_per_second;

        self.publish_history.put(messages_per_second as i32);
    }
}
