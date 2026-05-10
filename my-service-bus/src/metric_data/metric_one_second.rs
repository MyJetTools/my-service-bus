#[derive(Clone)]
pub struct MetricOneSecond {
    pub value: usize,
}

impl MetricOneSecond {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn increase(&mut self, delta: usize) {
        self.value += delta;
    }

    pub fn get_and_reset(&mut self) -> usize {
        let result = self.value;
        self.value = 0;
        result
    }
}
