use std::sync::Arc;

use my_service_bus::shared::sub_page::SubPageId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::{
    app::logs::Logs, grpc_client::MessagesPagesRepo, messages_page::SubPage, topics::Topic,
};

pub async fn load_page_to_cache(
    topic: &Arc<Topic>,
    messages_pages_repo: Arc<MessagesPagesRepo>,
    logs: Option<&Logs>,
    sub_page_id: SubPageId,
) -> SubPage {
    let mut dt = topic.restore_page_lock.lock().await;

    let sub_page =
        super::operations::load_page(topic.as_ref(), &messages_pages_repo, logs, sub_page_id).await;

    *dt = DateTimeAsMicroseconds::now();

    sub_page
}
