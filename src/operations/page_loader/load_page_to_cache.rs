use std::sync::Arc;

use my_service_bus::shared::sub_page::SubPageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{grpc_client::MessagesPagesRepo, messages_page::SubPage, topics::Topic};

pub async fn load_page_to_cache(
    topic: &Arc<Topic>,
    messages_pages_repo: Arc<MessagesPagesRepo>,

    sub_page_id: SubPageId,
) -> Option<SubPage> {
    let mut dt = topic.restore_page_lock.lock().await;

    {
        let topic_data = topic.get_access().await;
        if topic_data.pages.get(sub_page_id).is_some() {
            return None;
        }
    }

    let sub_page =
        super::operations::load_page(topic.as_ref(), &messages_pages_repo, sub_page_id).await;

    *dt = DateTimeAsMicroseconds::now();

    Some(sub_page)
}
