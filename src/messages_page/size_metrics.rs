pub struct SizeMetrics {
    pub messages_amount: usize,
    pub data_size: usize,
    pub persist_size: usize,
    pub avg_message_size: usize,
}

impl SizeMetrics {
    pub fn new(avg_message_size: usize) -> Self {
        Self {
            messages_amount: 0,
            data_size: 0,
            persist_size: 0,
            avg_message_size,
        }
    }

    pub fn append_without_avg(&mut self, other: &SizeMetrics) {
        self.messages_amount += other.messages_amount;
        self.data_size += other.data_size;
        self.persist_size += other.persist_size;
    }

    pub fn update(&mut self, data: &SizeMetrics) {
        self.messages_amount = data.messages_amount;
        self.data_size = data.data_size;
        self.persist_size = data.persist_size;
        self.avg_message_size = data.avg_message_size;
    }
}
