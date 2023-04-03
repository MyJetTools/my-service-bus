use std::collections::HashMap;

use my_service_bus_shared::sub_page::SubPageId;
use rust_extensions::lazy::LazyVec;

use crate::app::AppContext;
use crate::topics::TopicData;

pub fn gc_message_pages(_app: &AppContext, topic_data: &mut TopicData) {
    let active_pages = super::get_active_sub_pages(topic_data);

    let sub_pages_to_gc = get_subpages_to_gc(topic_data, &active_pages);

    if let Some(sub_pages_to_gc) = sub_pages_to_gc {
        for sub_page_to_gc in sub_pages_to_gc {
            let (_sub_page, _page) = topic_data.pages.gc_if_possible(sub_page_to_gc);

            #[cfg(test)]
            {
                if let Some(sub_page) = _sub_page {
                    println!(
                        "SubPage {} is GCed for topic: {}",
                        sub_page.sub_page_id.get_value(),
                        topic_data.topic_id.as_str()
                    );
                }

                if let Some(page) = _page {
                    println!(
                        "Page {} is GCed for topic: {}",
                        page.page_id.get_value(),
                        topic_data.topic_id.as_str()
                    );
                }
            }
        }
    }
}

fn get_subpages_to_gc(
    topic_data: &TopicData,
    active_pages: &HashMap<i64, SubPageId>,
) -> Option<Vec<SubPageId>> {
    let mut result = LazyVec::new();

    for page in topic_data.pages.get_pages() {
        for sub_page in page.sub_pages.values() {
            if !active_pages.contains_key(&sub_page.sub_page.sub_page_id.get_value()) {
                if sub_page.messages_to_persist.len() == 0 {
                    result.add(sub_page.sub_page.sub_page_id);
                }
            }
        }
    }

    result.get_result()
}
