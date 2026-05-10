use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus::shared::sub_page::SubPageId;

pub struct ActiveSubPages {
    pages: Vec<SubPageId>,
}

impl ActiveSubPages {
    pub fn new() -> Self {
        Self { pages: Vec::new() }
    }

    pub fn add_if_not_exists(&mut self, sub_page_id: SubPageId) {
        let index = self.pages.binary_search_by(|itm| itm.cmp(&sub_page_id));

        match index {
            Ok(_) => {}
            Err(index) => {
                self.pages.insert(index, sub_page_id);
            }
        }
    }

    pub fn add_intervals(&mut self, queue: &QueueWithIntervals) {
        for interval in queue.get_intervals() {
            if interval.is_empty() {
                continue;
            }
            let from = SubPageId::from_message_id(interval.from_id.into()).get_value();
            let to = SubPageId::from_message_id(interval.to_id.into()).get_value();
            for id in from..=to {
                self.add_if_not_exists(SubPageId::new(id));
            }
        }
    }

    pub fn has_sub_page(&self, sub_page_id: SubPageId) -> bool {
        let index = self.pages.binary_search_by(|itm| itm.cmp(&sub_page_id));
        index.is_ok()
    }

    pub fn as_slice(&self) -> &[SubPageId] {
        &self.pages
    }
}
