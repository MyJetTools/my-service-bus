use std::collections::{BTreeMap, HashMap};

use my_service_bus_abstractions::{queue_with_intervals::QueueWithIntervals, MessageId};
use my_service_bus_shared::{
    sub_page::{SubPage, SubPageId},
    MySbMessageContent,
};
use rust_extensions::lazy::LazyVec;

use super::MessagesPageMetrics;

pub struct MessagesPageList {
    pub pages: BTreeMap<SubPageId, SubPage>,
}

impl MessagesPageList {
    pub fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
        }
    }

    pub fn restore(&mut self, sub_page: SubPage) {
        self.pages.insert(sub_page.sub_page_id, sub_page);
    }

    pub fn get_sub_page(&self, sub_page_id: SubPageId) -> Option<&SubPage> {
        self.pages.get(&sub_page_id)
    }

    pub fn commit_persisted_messages(
        &mut self,
        sub_page_id: SubPageId,
        persisted: QueueWithIntervals,
    ) {
        if let Some(page) = self.pages.get_mut(&sub_page_id) {
            for msg_id in persisted {
                page.persisted(msg_id);
            }
        }
    }

    pub fn get_amount_to_persist(&self) -> i64 {
        let mut result = 0;
        for sub_page in self.pages.values() {
            result += sub_page.to_persist.len();
        }

        result
    }

    pub fn publish_brand_new_message(&mut self, content: MySbMessageContent) {
        let sub_page_id = SubPageId::from_message_id(content.id);

        if let Some(sub_page) = self.pages.get_mut(&sub_page_id) {
            sub_page.add_message(content);
            return;
        }

        let mut sub_page = SubPage::new(sub_page_id);
        sub_page.add_message(content);
        self.pages.insert(sub_page_id, sub_page);
    }

    pub fn get_persisted_min_message_id(&self) -> Option<MessageId> {
        for sub_page in self.pages.values() {
            if let Some(id) = sub_page.to_persist.peek() {
                return Some(id);
            }
        }

        None
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) {
        for page in self.pages.values_mut() {
            page.gc_messages(min_message_id);
        }
    }

    pub fn get_metrics(&self) -> MessagesPageMetrics {
        let mut result = MessagesPageMetrics {
            loaded_messages_amount: 0,
            content_size: 0,
            to_persist_size: 0,
        };
        for sub_page in self.pages.values() {
            let size_and_amount = sub_page.get_size_and_amount();
            result.loaded_messages_amount += size_and_amount.amount;
            result.content_size += size_and_amount.size;
            result.to_persist_size += sub_page.to_persist.len();
        }

        result
    }

    pub fn gc_sub_pages(&mut self, active_pages: &HashMap<SubPageId, ()>) {
        let mut to_gc = LazyVec::new();

        for sub_page in self.pages.values_mut() {
            if !active_pages.contains_key(&sub_page.sub_page_id) {
                to_gc.add(sub_page.sub_page_id);
            }
        }

        if let Some(to_gc) = to_gc.get_result() {
            for sub_page_id in to_gc {
                self.pages.remove(&sub_page_id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use my_service_bus_shared::{
        sub_page::{SubPage, SubPageId},
        MySbMessageContent,
    };
    use rust_extensions::date_time::DateTimeAsMicroseconds;

    use super::MessagesPageList;

    fn create_sub_page(sub_page_id: SubPageId) -> SubPage {
        let mut data = BTreeMap::new();
        for id in sub_page_id.iterate_message_ids() {
            let msg = MySbMessageContent {
                id,
                content: vec![],
                time: DateTimeAsMicroseconds::now(),
                headers: None,
            };

            data.insert(id, msg);
        }

        SubPage::restore(sub_page_id, data)
    }

    #[test]
    fn test_three_sub_pages_restore() {
        let mut pages_list = MessagesPageList::new();

        pages_list.restore(create_sub_page(SubPageId::new(0)));
        pages_list.restore(create_sub_page(SubPageId::new(1)));
        pages_list.restore(create_sub_page(SubPageId::new(3)));

        let metrics = pages_list.get_metrics();

        assert_eq!(3_000, metrics.loaded_messages_amount);
        assert_eq!(0, metrics.to_persist_size);
    }
}
