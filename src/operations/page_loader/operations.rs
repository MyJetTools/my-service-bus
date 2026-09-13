use std::{collections::BTreeMap, sync::Arc};

use my_service_bus::shared::sub_page::SubPageId;

use crate::{
    grpc_client::{PersistenceError, PersistenceGrpcService},
    messages_page::MySbCachedMessage,
    sub_page::{SubPage, SubPageInner},
    topics::Topic,
};

/// A single attempt. An error means persistence could not be reached or failed to answer - the
/// caller decides whether to repeat it (the restore events loop repeats the iteration).
pub async fn load_page(
    topic: &Topic,
    messages_pages_repo: &Arc<PersistenceGrpcService>,
    sub_page_id: SubPageId,
) -> Result<SubPage, PersistenceError> {
    let messages = messages_pages_repo
        .load_page(
            topic.as_grpc_namespace(),
            topic.topic_id.as_str(),
            sub_page_id.into(),
            sub_page_id.get_first_message_id(),
            sub_page_id.get_last_message_id(),
        )
        .await?;

    let Some(mut messages) = messages else {
        return Ok(SubPage::new_as_brand_new(sub_page_id));
    };

    let mut result = BTreeMap::new();
    for message_id in sub_page_id.iterate_message_ids() {
        if let Some(message) = messages.remove(&message_id) {
            result.insert(message_id, message.into());
        } else {
            result.insert(message_id, MySbCachedMessage::Missing(message_id.into()));
        }
    }

    let sub_page_inner = SubPageInner::restore(result);
    Ok(SubPage::restore(sub_page_id, sub_page_inner))
}
