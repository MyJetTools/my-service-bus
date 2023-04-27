use std::{
    collections::{BTreeMap, HashSet},
    time::Duration,
};

use my_service_bus_abstractions::MessageId;
use my_service_bus_shared::{page_id::PageId, sub_page::SubPageId};
use rust_extensions::{date_time::DateTimeAsMicroseconds, lazy::LazyVec};

use crate::utils::MinMessageIdCalculator;

use super::{MessagesToPersistBucket, PageSizeMetrics, SubPage, SubPageInner};

pub struct MessagesPageList {
    pub sub_pages: BTreeMap<i64, SubPage>,
}

impl MessagesPageList {
    pub fn new() -> Self {
        Self {
            sub_pages: BTreeMap::new(),
        }
    }

    pub fn get_or_create_mut(&mut self, sub_page_id: SubPageId) -> &mut SubPage {
        if !self.sub_pages.contains_key(sub_page_id.as_ref()) {
            let sub_page = SubPageInner::new(sub_page_id);
            self.sub_pages
                .insert(sub_page_id.get_value(), sub_page.into());
        }

        self.sub_pages.get_mut(sub_page_id.as_ref()).unwrap()
    }

    pub fn get(&self, sub_page_id: SubPageId) -> Option<&SubPage> {
        self.sub_pages.get(sub_page_id.as_ref())
    }

    pub fn restore_sub_page(&mut self, sub_page: SubPage) {
        self.sub_pages
            .insert(sub_page.get_id().get_value(), sub_page);
    }

    pub fn mark_messages_as_persisted(&mut self, bucket: &MessagesToPersistBucket) {
        if let Some(sub_page) = self.sub_pages.get_mut(bucket.sub_page_id.as_ref()) {
            sub_page.mark_messages_as_persisted(&bucket.ids);
        }
    }

    pub fn get_persisted_min_message_id(&self) -> Option<MessageId> {
        let mut min_message_id_calculator = MinMessageIdCalculator::new();

        for sub_page in self.sub_pages.values() {
            min_message_id_calculator.add(sub_page.get_min_message_to_persist());
        }

        min_message_id_calculator.get()
    }

    pub fn gc_pages(
        &mut self,
        active_pages: &HashSet<i64>,
        now: DateTimeAsMicroseconds,
        gc_delay: Duration,
    ) {
        let pages_to_gc = self.get_sub_pages_to_gc(active_pages, now, gc_delay);

        if let Some(pages_to_gc) = pages_to_gc {
            for page_id in pages_to_gc {
                self.sub_pages.remove(page_id.as_ref());
            }
        }
    }

    pub fn gc_messages(
        &mut self,
        min_message_id: MessageId,
        now: DateTimeAsMicroseconds,
        gc_delay: Duration,
    ) {
        let mut pages_to_gc = LazyVec::new();

        for page in self.sub_pages.values_mut() {
            if page.gc_messages(min_message_id) {
                if page.ready_to_be_gc(now, gc_delay) {
                    pages_to_gc.add(page.get_id());
                }
            }
        }

        if let Some(pages_to_gc) = pages_to_gc.get_result() {
            for sub_page_id in pages_to_gc {
                self.sub_pages.remove(sub_page_id.as_ref());
            }
        }
    }

    pub fn get_sub_pages_to_gc(
        &self,
        active_pages: &HashSet<i64>,
        now: DateTimeAsMicroseconds,
        gc_delay: Duration,
    ) -> Option<Vec<SubPageId>> {
        let mut result = LazyVec::new();

        for sub_page in self.sub_pages.values() {
            let sub_page_id = sub_page.get_id();
            if !active_pages.contains(sub_page_id.as_ref()) {
                if sub_page.ready_to_be_gc(now, gc_delay) {
                    result.add(sub_page_id);
                }
            }
        }

        result.get_result()
    }

    pub fn get_messages_to_persist(&self, max_size: usize) -> Option<MessagesToPersistBucket> {
        for sub_page in self.sub_pages.values() {
            if let Some(messages_to_persist) = sub_page.get_messages_to_persist(max_size) {
                return Some(messages_to_persist);
            }
        }

        None
    }

    pub fn get_page_size_metrics(&self) -> BTreeMap<i64, PageSizeMetrics> {
        let mut result: BTreeMap<i64, PageSizeMetrics> = BTreeMap::new();

        for sub_page in self.sub_pages.values() {
            let page_id: PageId = sub_page.get_id().into();
            let size_metrics = sub_page.get_size_metrics();

            if let Some(itm) = result.get_mut(page_id.as_ref()) {
                itm.sub_page_metrics
                    .insert(sub_page.get_id().get_value(), size_metrics);
            } else {
                result.insert(
                    page_id.get_value(),
                    PageSizeMetrics::new(sub_page.get_id().get_value(), size_metrics),
                );
            }
        }

        result
    }
}
