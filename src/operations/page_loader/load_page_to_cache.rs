use std::sync::Arc;

use my_service_bus_shared::{
    page_id::PageId,
    sub_page::{SubPage, SubPageId},
};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{app::logs::Logs, grpc_client::MessagesPagesRepo, topics::Topic};

pub async fn load_page_to_cache(
    topic: &Arc<Topic>,
    messages_pages_repo: Arc<MessagesPagesRepo>,
    logs: Option<&Logs>,
    page_id: PageId,
    sub_page_id: SubPageId,
) -> SubPage {
    let mut dt = topic.restore_page_lock.lock().await;

    let from_message_id = sub_page_id.get_first_message_id();
    let to_message_id = sub_page_id
        .get_first_message_id_of_next_sub_page()
        .get_value()
        - 1;

    println!(
        "Loading messages {}-{} for page {} with sub_page {} for topic:{}",
        from_message_id,
        to_message_id,
        page_id.get_value(),
        sub_page_id.get_value(),
        topic.topic_id
    );

    let sub_page = super::operations::load_page(
        topic.as_ref(),
        &messages_pages_repo,
        logs,
        page_id,
        sub_page_id,
    )
    .await;

    *dt = DateTimeAsMicroseconds::now();

    sub_page
}
