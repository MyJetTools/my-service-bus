use std::sync::Arc;

use mcp_server_middleware::*;
use my_service_bus::shared::page_id::PageId;
use my_service_bus::shared::sub_page::SubPageId;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct PersistenceLoadPageInput {
    #[property(description = "Topic id to request from the persistence service")]
    pub topic_id: String,
    #[property(
        description = "Sub-page id to request (as reported by mysb_get_topic_pages). Holds up to 1000 consecutive message ids"
    )]
    pub sub_page_id: i64,
    #[property(description = "Namespace to work in. Optional, absent means the default namespace")]
    pub namespace: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct PersistenceLoadPageResponse {
    #[property(description = "Topic id")]
    pub topic_id: String,
    #[property(description = "Sub-page id that was requested")]
    pub sub_page_id: i64,
    #[property(description = "Page number (100k-message page) sent to persistence in the get_page request")]
    pub page_no: i64,
    #[property(description = "First message id of the requested range")]
    pub from_message_id: i64,
    #[property(description = "Last message id of the requested range")]
    pub to_message_id: i64,
    #[property(
        description = "true when persistence returned a page for this range; false when persistence has no such page at all"
    )]
    pub page_present: bool,
    #[property(description = "Number of messages persistence actually returned inside the requested range")]
    pub messages_returned: usize,
    #[property(description = "Sum of payload sizes (bytes) of the messages returned by persistence")]
    pub total_size_bytes: usize,
    #[property(description = "Number of message id slots the requested range can hold")]
    pub range_capacity: usize,
    #[property(description = "Message ids of the range that persistence did NOT return (range_capacity - messages_returned)")]
    pub missing_in_range: usize,
    #[property(description = "Lowest message id persistence returned, null when nothing was returned")]
    pub first_returned_message_id: Option<i64>,
    #[property(description = "Highest message id persistence returned, null when nothing was returned")]
    pub last_returned_message_id: Option<i64>,
    #[property(description = "Human readable summary of what persistence returned")]
    pub summary: String,
}

pub struct PersistenceLoadPageHandler {
    app: Arc<AppContext>,
}

impl PersistenceLoadPageHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for PersistenceLoadPageHandler {
    const FUNC_NAME: &'static str = "mysb_persistence_load_page";
    const DESCRIPTION: &'static str =
        "Requests a message sub-page from the persistence service over gRPC using the SAME get_page call the broker issues when it fills a delivery packet (restoring a cold/GC-ed sub-page). Reports whether persistence returned content, how many messages and bytes came back, and how many ids in the range are missing. Use it to tell whether a stuck sub-page can actually be read back from persistence, i.e. whether the problem is on the persistence side or the broker side.";
}

#[async_trait::async_trait]
impl McpToolCall<PersistenceLoadPageInput, PersistenceLoadPageResponse> for PersistenceLoadPageHandler {
    async fn execute_tool_call(
        &self,
        model: PersistenceLoadPageInput,
    ) -> Result<PersistenceLoadPageResponse, String> {
        let namespace = self
            .app
            .namespaces
            .get_or_create_optional(model.namespace.as_deref())
            .map_err(|err| format!("Invalid namespace. {}", err))?;

        let sub_page_id = SubPageId::new(model.sub_page_id);
        let page_id: PageId = sub_page_id.into();
        let from_message_id = sub_page_id.get_first_message_id();
        let to_message_id = sub_page_id.get_last_message_id();

        let page_no = page_id.get_value();
        let from_value = from_message_id.get_value();
        let to_value = to_message_id.get_value();
        let range_capacity = (to_value - from_value + 1).max(0) as usize;

        let result = self
            .app
            .persistence_client
            .load_page(
                namespace.as_grpc_namespace(),
                &model.topic_id,
                page_id,
                from_message_id,
                to_message_id,
            )
            .await
            .map_err(|err| {
                format!(
                    "Failed to load sub-page {} (page {}) of topic '{}' from persistence: {:?}",
                    model.sub_page_id, page_no, model.topic_id, err
                )
            })?;

        let response = match result {
            None => PersistenceLoadPageResponse {
                topic_id: model.topic_id.clone(),
                sub_page_id: model.sub_page_id,
                page_no,
                from_message_id: from_value,
                to_message_id: to_value,
                page_present: false,
                messages_returned: 0,
                total_size_bytes: 0,
                range_capacity,
                missing_in_range: range_capacity,
                first_returned_message_id: None,
                last_returned_message_id: None,
                summary: format!(
                    "No content: persistence has no page for sub-page {} (page {}, ids {}..{}) of topic '{}'.",
                    model.sub_page_id, page_no, from_value, to_value, model.topic_id
                ),
            },
            Some(messages) => {
                let messages_returned = messages.len();
                let total_size_bytes: usize = messages.values().map(|m| m.content.len()).sum();
                let first_returned_message_id = messages.keys().next().copied();
                let last_returned_message_id = messages.keys().next_back().copied();
                let missing_in_range = range_capacity.saturating_sub(messages_returned);

                let summary = format!(
                    "Content present: {} messages, {} bytes returned for ids {}..{}; {} of {} ids missing within the range.",
                    messages_returned,
                    total_size_bytes,
                    first_returned_message_id.unwrap_or(0),
                    last_returned_message_id.unwrap_or(0),
                    missing_in_range,
                    range_capacity,
                );

                PersistenceLoadPageResponse {
                    topic_id: model.topic_id.clone(),
                    sub_page_id: model.sub_page_id,
                    page_no,
                    from_message_id: from_value,
                    to_message_id: to_value,
                    page_present: true,
                    messages_returned,
                    total_size_bytes,
                    range_capacity,
                    missing_in_range,
                    first_returned_message_id,
                    last_returned_message_id,
                    summary,
                }
            }
        };

        Ok(response)
    }
}
