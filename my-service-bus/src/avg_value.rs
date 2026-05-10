pub struct AvgValue {
    pub value: usize,
    pub count: usize,
}

impl AvgValue {
    pub fn new() -> Self {
        AvgValue { value: 0, count: 0 }
    }

    pub fn add(&mut self, value: usize) {
        self.value += value;
        self.count += 1;
    }

    pub fn get(&self) -> usize {
        if self.count == 0 {
            return 0;
        }

        self.value / self.count
    }
}
