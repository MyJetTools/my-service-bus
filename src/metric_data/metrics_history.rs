use std::collections::VecDeque;

#[derive(Clone)]
pub struct MetricsHistory {
    data: VecDeque<i32>,
}

const MAX_DATA_LEN: usize = 120;
impl MetricsHistory {
    pub fn new() -> Self {
        Self {
            data: VecDeque::with_capacity(MAX_DATA_LEN),
        }
    }

    pub fn put(&mut self, value: i32) {
        if self.data.len() == MAX_DATA_LEN {
            self.data.remove(0);
        }
        self.data.push_back(value);
    }

    pub fn get(&self) -> Vec<i32> {
        self.data.iter().map(|itm| itm.clone()).collect()
    }
}
