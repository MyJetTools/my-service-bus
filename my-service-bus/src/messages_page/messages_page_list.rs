use std::collections::BTreeMap;

use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;
use my_service_bus::shared::{page_id::PageId, sub_page::SubPageId};
use rust_extensions::sorted_vec::{GetMutOrCreateEntry, SortedVec};

use super::*;
use crate::sub_page::*;

pub struct MessagesPageList {
    pub sub_pages: SortedVec<i64, SubPage>,
}

impl MessagesPageList {
    pub fn new() -> Self {
        Self {
            sub_pages: SortedVec::new(),
        }
    }

    pub fn get_or_create_mut(&mut self, sub_page_id: SubPageId) -> &mut SubPage {
        match self.sub_pages.get_mut_or_create(sub_page_id.as_ref()) {
            GetMutOrCreateEntry::GetMut(item) => return item,
            GetMutOrCreateEntry::Create(entry) => {
                let sub_page = SubPage::new_as_brand_new(sub_page_id);
                entry.insert_and_get_value_mut(sub_page)
            }
        }
    }

    #[cfg(test)]
    pub fn get(&self, sub_page_id: SubPageId) -> Option<&SubPage> {
        self.sub_pages.get(sub_page_id.as_ref())
    }

    pub fn get_mut(&mut self, sub_page_id: SubPageId) -> Option<&mut SubPage> {
        self.sub_pages.get_mut(sub_page_id.as_ref())
    }

    pub fn restore_sub_page(&mut self, sub_page: SubPage) {
        self.sub_pages.insert_or_replace(sub_page);
    }

    /*
        pub fn delete_sub_page(&mut self, sub_page_id: SubPageId) {
            self.sub_pages.remove(sub_page_id.as_ref());
        }
    */
    pub fn mark_messages_as_persisted(&mut self, sub_page_id: SubPageId, ids: &QueueWithIntervals) {
        if let Some(sub_page) = self.sub_pages.get_mut(sub_page_id.as_ref()) {
            sub_page.mark_messages_as_persisted(ids);
        }
    }

    pub fn gc_pages(&mut self, active_pages: &ActiveSubPages) {
        let pages_to_gc = self.get_sub_pages_to_gc(active_pages);

        for page_id in pages_to_gc {
            self.sub_pages.remove(page_id.as_ref());
        }
    }

    pub fn find_next_existing_sub_page(&self, after: SubPageId) -> Option<SubPageId> {
        let after = after.get_value();
        for sub_page in self.sub_pages.iter() {
            let id = sub_page.get_id().get_value();
            if id > after {
                return Some(sub_page.get_id());
            }
        }
        None
    }

    pub fn gc_all_except(&mut self, keep: SubPageId) {
        let mut to_remove = Vec::new();

        for sub_page in self.sub_pages.iter() {
            if sub_page.get_id().get_value() == keep.get_value() {
                continue;
            }

            if !sub_page.is_ready_to_gc(&[]) {
                continue;
            }

            to_remove.push(sub_page.get_id());
        }

        for page_id in to_remove {
            self.sub_pages.remove(page_id.as_ref());
        }
    }

    pub fn gc_messages(
        &mut self,
        min_message_id: my_service_bus::abstractions::MessageId,
        active_sub_pages: &ActiveSubPages,
    ) {
        let mut pages_to_gc = Vec::new();

        for sub_page in self.sub_pages.iter_mut() {
            sub_page.gc_messages(min_message_id);

            if sub_page.is_empty() {
                if !active_sub_pages.has_sub_page(sub_page.get_id()) {
                    pages_to_gc.push(sub_page.get_id());
                }
            }
        }

        for sub_page_id in pages_to_gc {
            self.sub_pages.remove(sub_page_id.as_ref());
        }
    }

    pub fn get_sub_pages_to_gc(&self, active_pages: &ActiveSubPages) -> Vec<SubPageId> {
        let mut result = Vec::new();

        for sub_page in self.sub_pages.iter() {
            let sub_page_id = sub_page.get_id();

            if sub_page.is_ready_to_gc(active_pages.as_slice()) {
                result.push(sub_page_id);
            }
        }

        result
    }

    pub fn get_messages_to_persist<TResult>(
        &self,
        result: &mut Vec<(SubPageId, Vec<TResult>)>,
        transform: impl Fn(&MySbMessageContent) -> TResult,
    ) {
        for sub_page in self.sub_pages.iter() {
            sub_page.get_messages_to_persist(result, &transform)
        }
    }

    #[cfg(test)]
    pub fn has_sub_page_in_cache(&self, sub_page_id: SubPageId) -> bool {
        self.sub_pages.get(sub_page_id.as_ref()).is_some()
    }

    pub fn get_page_size_metrics(&self) -> BTreeMap<i64, PageSizeMetrics> {
        let mut result: BTreeMap<i64, PageSizeMetrics> = BTreeMap::new();

        for sub_page in self.sub_pages.iter() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_next_existing_sub_page_returns_strictly_greater() {
        let mut list = MessagesPageList::new();

        list.restore_sub_page(SubPage::new_as_brand_new(SubPageId::new(1)));
        list.restore_sub_page(SubPage::new_as_brand_new(SubPageId::new(5)));
        list.restore_sub_page(SubPage::new_as_brand_new(SubPageId::new(10)));

        assert_eq!(
            list.find_next_existing_sub_page(SubPageId::new(0))
                .map(|p| p.get_value()),
            Some(1)
        );
        assert_eq!(
            list.find_next_existing_sub_page(SubPageId::new(1))
                .map(|p| p.get_value()),
            Some(5)
        );
        assert_eq!(
            list.find_next_existing_sub_page(SubPageId::new(5))
                .map(|p| p.get_value()),
            Some(10)
        );
        assert_eq!(
            list.find_next_existing_sub_page(SubPageId::new(10))
                .map(|p| p.get_value()),
            None
        );
        assert_eq!(
            list.find_next_existing_sub_page(SubPageId::new(20))
                .map(|p| p.get_value()),
            None
        );
    }

    #[test]
    fn find_next_existing_sub_page_empty_returns_none() {
        let list = MessagesPageList::new();

        assert_eq!(
            list.find_next_existing_sub_page(SubPageId::new(0))
                .map(|p| p.get_value()),
            None
        );
    }
}
