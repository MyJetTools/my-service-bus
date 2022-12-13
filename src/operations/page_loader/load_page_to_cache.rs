use std::sync::Arc;

use my_service_bus_shared::sub_page::SubPageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{persistence::MessagesPagesRepo, topics::Topic};

pub async fn load_page_to_cache(
    topic: Arc<Topic>,
    messages_pages_repo: Arc<MessagesPagesRepo>,
    sub_page_id: SubPageId,
) {
    let mut dt = topic.restore_page_lock.lock().await;

    let from_message_id = sub_page_id.get_first_message_id();
    let to_message_id = sub_page_id.get_first_message_id();

    println!(
        "Loading messages {}-{} for subpage {} for topic:{}",
        from_message_id,
        to_message_id,
        sub_page_id.get_value(),
        topic.topic_id
    );

    let sub_page =
        super::operations::load_page(topic.as_ref(), &messages_pages_repo, sub_page_id).await;

    let mut topic_data = topic.get_access().await;
    topic_data.pages.restore(sub_page);

    *dt = DateTimeAsMicroseconds::now();
}
