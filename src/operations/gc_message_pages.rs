use crate::topics::TopicData;

pub fn gc_message_pages(topic_data: &mut TopicData) {
    let active_pages = topic_data.get_active_sub_pages();

    let sub_pages_to_gc = topic_data.get_sub_pages_to_gc(&active_pages);

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
