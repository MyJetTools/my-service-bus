use std::collections::BTreeMap;

use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::{
    page_id::PageId,
    sub_page::{SubPage, SubPageId},
    MySbMessageContent,
};

use crate::utils::MinMessageIdCalculator;

use super::{PageSizeMetrics, SubPageData};

pub struct MessagesPage {
    pub page_id: PageId,
    pub sub_pages: BTreeMap<usize, SubPageData>,
}

impl MessagesPage {
    pub fn new(page_id: PageId) -> Self {
        Self {
            page_id,
            sub_pages: BTreeMap::new(),
        }
    }

    pub fn get_or_create_sub_page(&mut self, sub_page_id: SubPageId) -> &mut SubPageData {
        if !self.sub_pages.contains_key(&sub_page_id.value) {
            let sub_page = SubPageData::new(SubPage::new(sub_page_id));
            self.sub_pages.insert(sub_page_id.value, sub_page);
        }

        self.sub_pages.get_mut(&sub_page_id.value).unwrap()
    }

    pub fn publish_message(&mut self, message: MySbMessageContent) {
        let sub_page_id = SubPageId::from_message_id(message.id);

        let sub_page = self.get_or_create_sub_page(sub_page_id);
        sub_page.messages_to_persist.enqueue(message.id);
        sub_page.sub_page.add_message(message);
    }

    pub fn get_sub_page(&self, sub_page_id: &SubPageId) -> Option<&SubPageData> {
        self.sub_pages.get(&sub_page_id.value)
    }

    pub fn get_sub_page_mut(&mut self, sub_page_id: &SubPageId) -> Option<&mut SubPageData> {
        self.sub_pages.get_mut(&sub_page_id.value)
    }

    pub fn add_sub_page(&mut self, sub_page: SubPage) {
        self.sub_pages
            .insert(sub_page.sub_page_id.value, SubPageData::new(sub_page));
    }

    pub fn get_page_size_metrics(&self) -> PageSizeMetrics {
        let mut result = PageSizeMetrics::new();

        for sub_page_data in self.sub_pages.values() {
            let size_and_amount = sub_page_data.sub_page.get_size_and_amount();
            result.messages_amount += size_and_amount.amount;
            result.data_size += size_and_amount.size;
            result.persist_size += sub_page_data.messages_to_persist.len() as usize;
        }

        result
    }

    pub fn get_sub_page_with_messages_to_persist(&mut self) -> Option<&mut SubPageData> {
        for sub_page in self.sub_pages.values_mut() {
            if sub_page.messages_to_persist.len() > 0 {
                return Some(sub_page);
            }
        }

        None
    }

    pub fn gc_if_possible(&mut self, sub_page_id: &SubPageId) -> Option<SubPage> {
        if let Some(sub_page) = self.sub_pages.get(&sub_page_id.value) {
            if !sub_page.can_be_gced() {
                return None;
            }
        } else {
            return None;
        }

        let result = self.sub_pages.remove(&sub_page_id.value)?;

        Some(result.sub_page)
    }

    pub fn sub_pages_amount(&self) -> usize {
        self.sub_pages.len()
    }

    pub fn get_sub_pages(&self) -> Vec<usize> {
        let sub_page_id = SubPageId::from_message_id(
            self.page_id * my_service_bus_shared::page_id::MESSAGES_IN_PAGE,
        );

        self.sub_pages
            .keys()
            .map(|itm| *itm - sub_page_id.value)
            .collect()
    }

    pub fn get_persisted_min_message_id(&self) -> Option<MessageId> {
        let mut min_message_id_calculator = MinMessageIdCalculator::new();

        for page in self.sub_pages.values() {
            min_message_id_calculator.add(page.messages_to_persist.get_min_id());
        }

        min_message_id_calculator.value
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId) {
        for page in self.sub_pages.values_mut() {
            page.sub_page.gc_messages(min_message_id);
        }
    }
}
