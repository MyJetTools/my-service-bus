use std::collections::HashMap;

use my_service_bus_shared::sub_page::SubPageId;

use crate::topics::TopicData;

pub fn get_active_sub_pages(topic_data: &TopicData) -> HashMap<i64, SubPageId> {
    let mut result: HashMap<i64, SubPageId> = HashMap::new();

    let sub_page_id: SubPageId = topic_data.message_id.into();

    result.insert(sub_page_id.get_value(), sub_page_id);

    for queue in topic_data.queues.get_all() {
        if let Some(min_msg_id) = queue.get_min_msg_id() {
            let sub_page_id = SubPageId::from_message_id(min_msg_id);

            if !result.contains_key(&sub_page_id.get_value()) {
                result.insert(sub_page_id.get_value(), sub_page_id);
            }
        }
    }

    result
}
