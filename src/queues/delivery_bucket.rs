use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;

pub struct DeliveryBucket {
    pub ids: QueueWithIntervals,
    pub confirmed: usize,
}

impl DeliveryBucket {
    pub fn new(ids: QueueWithIntervals) -> Self {
        Self { ids, confirmed: 0 }
    }

    pub fn confirmed(&mut self, confirmed: &QueueWithIntervals) {
        self.confirmed += confirmed.len() as usize;

        for id in confirmed {
            if let Err(err) = self.ids.remove(id) {
                println!(
                    "We are trying to confirm message {} - but something went wrong. Reason: {:?}",
                    id, err
                )
            }
        }
    }
}
