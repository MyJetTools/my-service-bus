use std::collections::BTreeMap;

use super::SizeMetrics;

pub struct PageSizeMetrics {
    pub sub_page_metrics: BTreeMap<i64, SizeMetrics>,
}

impl PageSizeMetrics {
    pub fn new(sub_page_id: i64, metrics: SizeMetrics) -> Self {
        let mut sub_page_metrics = BTreeMap::new();
        sub_page_metrics.insert(sub_page_id, metrics);

        Self { sub_page_metrics }
    }

    pub fn get_sub_pages(&self) -> impl Iterator<Item = &i64> {
        self.sub_page_metrics.keys()
    }

    pub fn get_size_metrics(&self) -> SizeMetrics {
        let mut result = SizeMetrics::new(0);

        for metrics in self.sub_page_metrics.values() {
            result.append(metrics);
        }

        result
    }
}
