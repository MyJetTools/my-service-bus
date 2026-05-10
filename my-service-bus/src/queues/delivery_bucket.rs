use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;

#[derive(Debug)]
pub struct DeliveryBucket {
    pub to_be_confirmed: QueueWithIntervals,
    pub confirmed: QueueWithIntervals,
}

impl DeliveryBucket {
    pub fn new(ids: QueueWithIntervals) -> Self {
        Self {
            to_be_confirmed: ids,
            confirmed: QueueWithIntervals::new(),
        }
    }

    pub fn confirmed(&mut self, confirmed: &QueueWithIntervals) {
        self.confirmed.merge(confirmed.clone());

        for id in confirmed {
            let _ = self.to_be_confirmed.remove(id);
        }
    }
}
