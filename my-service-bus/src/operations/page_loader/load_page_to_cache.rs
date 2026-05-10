use std::sync::Arc;

use my_service_bus::shared::sub_page::SubPageId;

use crate::{grpc_client::PersistenceGrpcService, topics::Topic};

pub async fn load_page_to_cache(
    topic: &Arc<Topic>,
    messages_pages_repo: &Arc<PersistenceGrpcService>,
    sub_page_id: SubPageId,
) {
    let sub_page =
        super::operations::load_page(topic.as_ref(), &messages_pages_repo, sub_page_id).await;

    let mut topic_data = topic.get_access();
    topic_data.pages.restore_sub_page(sub_page);
}
