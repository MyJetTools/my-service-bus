use std::collections::BTreeMap;
use std::sync::Arc;

use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::{page_id::PageId, sub_page::SubPageId};

use super::{ActiveSubPages, MySbMessageContent, PageSizeMetrics, SubPage, SubPageInner};

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

    pub fn delete_sub_page(&mut self, sub_page_id: SubPageId) {
        self.sub_pages.remove(sub_page_id.as_ref());
    }

    pub fn mark_messages_as_persisted(&mut self, sub_page_id: SubPageId, ids: &QueueWithIntervals) {
        if let Some(sub_page) = self.sub_pages.get_mut(sub_page_id.as_ref()) {
            sub_page.mark_messages_as_persisted(ids);
        }
    }

    pub fn gc_pages(&mut self, active_pages: &ActiveSubPages, min_message_id: MessageId) {
        let pages_to_gc = self.get_sub_pages_to_gc(active_pages, min_message_id);

        for page_id in pages_to_gc {
            self.sub_pages.remove(page_id.as_ref());
        }
    }

    pub fn gc_messages(&mut self, min_message_id: MessageId, current_sub_page_id: SubPageId) {
        let mut pages_to_gc = Vec::new();

        for page in self.sub_pages.values_mut() {
            page.gc_messages(min_message_id);

            if page.is_empty() {
                if current_sub_page_id.get_value() != page.get_id().get_value() {
                    pages_to_gc.push(page.get_id());
                }
            }
        }

        for sub_page_id in pages_to_gc {
            self.sub_pages.remove(sub_page_id.as_ref());
        }
    }

    pub fn get_sub_pages_to_gc(
        &self,
        active_pages: &ActiveSubPages,
        min_message_id: MessageId,
    ) -> Vec<SubPageId> {
        let mut result = Vec::new();

        for sub_page in self.sub_pages.values() {
            let sub_page_id = sub_page.get_id();
            if !active_pages.has_sub_page(sub_page_id) {
                if sub_page.is_ready_to_be_gc(min_message_id) {
                    result.push(sub_page_id);
                }
            }
        }

        result
    }

    pub fn get_messages_to_persist(
        &self,
        result: &mut Vec<(SubPageId, Vec<Arc<MySbMessageContent>>)>,
    ) {
        for sub_page in self.sub_pages.values() {
            sub_page.get_messages_to_persist(result)
        }
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
