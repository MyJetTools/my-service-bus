use std::collections::{
    btree_map::{Values, ValuesMut},
    BTreeMap,
};

use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::{
    page_id::{get_page_id, PageId},
    sub_page::{SubPage, SubPageId},
};

use crate::utils::MinMessageIdCalculator;

use super::{messages_page::MessagesPage, MessagesToPersistBucket, PageSizeMetrics};

pub struct MessagesPageList {
    pub pages: BTreeMap<PageId, MessagesPage>,
}

impl MessagesPageList {
    pub fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
        }
    }

    pub fn get_or_create_page_mut(&mut self, page_id: PageId) -> &mut MessagesPage {
        if !self.pages.contains_key(&page_id) {
            let page = MessagesPage::new(page_id);
            self.pages.insert(page_id, page);
        }

        self.pages.get_mut(&page_id).unwrap()
    }

    pub fn get_page(&self, page_id: PageId) -> Option<&MessagesPage> {
        self.pages.get(&page_id)
    }

    pub fn get_page_size_metrics(&self) -> PageSizeMetrics {
        let mut result = PageSizeMetrics::new();

        for page in self.pages.values() {
            let page_size_metrics = page.get_page_size_metrics();
            result.append(&page_size_metrics);
        }

        result
    }

    pub fn get_pages(&self) -> Values<PageId, MessagesPage> {
        self.pages.values()
    }

    pub fn get_pages_mut(&mut self) -> ValuesMut<PageId, MessagesPage> {
        self.pages.values_mut()
    }

    pub fn restore_subpage(&mut self, sub_page: SubPage) {
        let first_message_id = sub_page.sub_page_id.get_first_message_id();
        let page_id = get_page_id(first_message_id);

        if let Some(page) = self.pages.get_mut(&page_id) {
            page.add_sub_page(sub_page);
        } else {
            let mut page = MessagesPage::new(page_id);
            page.add_sub_page(sub_page);
            self.pages.insert(page_id, page);
        }
    }

    pub fn commit_persisted_messages(
        &mut self,
        sub_page_id: SubPageId,
        messages_to_persist: &MessagesToPersistBucket,
        persisted: bool,
    ) {
        let page_id = get_page_id(sub_page_id.get_first_message_id());

        if let Some(page) = self.pages.get_mut(&page_id) {
            if let Some(sub_page_data) = page.get_sub_page_mut(&sub_page_id) {
                sub_page_data.commit_persisted_messages(messages_to_persist, persisted);
            }
        }
    }

    pub fn get_persisted_min_message_id(&self) -> Option<MessageId> {
        let mut min_message_id_calculator = MinMessageIdCalculator::new();

        for page in self.pages.values() {
            min_message_id_calculator.add(page.get_persisted_min_message_id());
        }

        min_message_id_calculator.value
    }

    pub fn gc_if_possible(
        &mut self,
        sub_page_id: SubPageId,
    ) -> (Option<SubPage>, Option<MessagesPage>) {
        let page_id = get_page_id(sub_page_id.get_first_message_id());

        let (gced_sub_page, remove_page) = {
            if let Some(page) = self.pages.get_mut(&page_id) {
                let result = page.gc_if_possible(&sub_page_id);
                let remove_page = page.sub_pages_amount() == 0;

                (result, remove_page)
            } else {
                return (None, None);
            }
        };

        let gced_page = if remove_page {
            self.pages.remove(&page_id)
        } else {
            None
        };

        (gced_sub_page, gced_page)
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) {
        for page in self.pages.values_mut() {
            page.gc_messages(min_message_id);
        }
    }
}
