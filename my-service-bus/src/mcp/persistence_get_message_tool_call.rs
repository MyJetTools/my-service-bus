use std::sync::Arc;

use mcp_server_middleware::*;
use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::page_id::PageId;
use my_service_bus::shared::sub_page::SubPageId;
use rust_extensions::base64::IntoBase64;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

use super::MessageHeaderView;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct PersistenceGetMessageInput {
    #[property(description = "Topic id the message belongs to")]
    pub topic_id: String,
    #[property(description = "Message id to fetch from the persistence service")]
    pub message_id: i64,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct PersistenceGetMessageResponse {
    #[property(description = "Topic id")]
    pub topic_id: String,
    #[property(description = "Message id")]
    pub message_id: i64,
    #[property(description = "Sub-page id this message id maps to")]
    pub sub_page_id: i64,
    #[property(description = "Page number (100k-message page) sent to persistence in the get_page request")]
    pub page_no: i64,
    #[property(
        description = "true when persistence returned the page that should contain this message; false when persistence has no such page at all"
    )]
    pub page_present: bool,
    #[property(description = "true when the message payload was found inside the returned page")]
    pub found: bool,
    #[property(description = "Payload size in bytes (0 when not found)")]
    pub size: usize,
    #[property(description = "Creation time as unix microseconds (0 when not found)")]
    pub created_unix_microseconds: i64,
    #[property(
        description = "Payload decoded as UTF-8. Null when not found or when the payload is not valid UTF-8"
    )]
    pub content_text: Option<String>,
    #[property(description = "Payload encoded as Base64. Null when not found")]
    pub content_base64: Option<String>,
    #[property(description = "Message headers (empty when not found)")]
    pub headers: Vec<MessageHeaderView>,
    #[property(description = "Human readable summary of what persistence returned")]
    pub summary: String,
}

pub struct PersistenceGetMessageHandler {
    app: Arc<AppContext>,
}

impl PersistenceGetMessageHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for PersistenceGetMessageHandler {
    const FUNC_NAME: &'static str = "mysb_persistence_get_message";
    const DESCRIPTION: &'static str =
        "Fetches a single message by id from the persistence service over gRPC via the get_page (delivery-packet) path: it requests the whole sub-page the broker would restore and extracts the requested message from it. This complements mysb_get_message (which uses the single-message get_message call) - comparing the two tells you whether a persistence problem is specific to the page path the delivery loop relies on. Returns the payload (UTF-8 when valid, plus Base64), headers and creation time.";
}

#[async_trait::async_trait]
impl McpToolCall<PersistenceGetMessageInput, PersistenceGetMessageResponse> for PersistenceGetMessageHandler {
    async fn execute_tool_call(
        &self,
        model: PersistenceGetMessageInput,
    ) -> Result<PersistenceGetMessageResponse, String> {
        let message_id: MessageId = model.message_id.into();
        let sub_page_id: SubPageId = message_id.into();
        let page_id: PageId = sub_page_id.into();

        let sub_page_id_value = sub_page_id.get_value();
        let page_no = page_id.get_value();

        let result = self
            .app
            .persistence_client
            .load_page(
                &model.topic_id,
                page_id,
                sub_page_id.get_first_message_id(),
                sub_page_id.get_last_message_id(),
            )
            .await
            .map_err(|err| {
                format!(
                    "Failed to load page {} (for message {}) of topic '{}' from persistence: {:?}",
                    page_no, model.message_id, model.topic_id, err
                )
            })?;

        let response = match result {
            None => PersistenceGetMessageResponse {
                topic_id: model.topic_id.clone(),
                message_id: model.message_id,
                sub_page_id: sub_page_id_value,
                page_no,
                page_present: false,
                found: false,
                size: 0,
                created_unix_microseconds: 0,
                content_text: None,
                content_base64: None,
                headers: Vec::new(),
                summary: format!(
                    "No content: persistence has no page (page {}) for message {} of topic '{}'.",
                    page_no, model.message_id, model.topic_id
                ),
            },
            Some(messages) => match messages.get(&model.message_id) {
                Some(content) => {
                    let size = content.content.len();
                    let content_text = std::str::from_utf8(&content.content)
                        .ok()
                        .map(|s| s.to_string());
                    let content_base64 = content.content.as_slice().into_base64();
                    let headers = content
                        .headers
                        .iter()
                        .map(|(name, value)| MessageHeaderView {
                            name: name.to_string(),
                            text: value.to_string(),
                        })
                        .collect();

                    PersistenceGetMessageResponse {
                        topic_id: model.topic_id.clone(),
                        message_id: model.message_id,
                        sub_page_id: sub_page_id_value,
                        page_no,
                        page_present: true,
                        found: true,
                        size,
                        created_unix_microseconds: content.time.unix_microseconds,
                        content_text,
                        content_base64: Some(content_base64),
                        headers,
                        summary: format!(
                            "Found: message {} of topic '{}' returned {} bytes from persistence via the page path.",
                            model.message_id, model.topic_id, size
                        ),
                    }
                }
                None => PersistenceGetMessageResponse {
                    topic_id: model.topic_id.clone(),
                    message_id: model.message_id,
                    sub_page_id: sub_page_id_value,
                    page_no,
                    page_present: true,
                    found: false,
                    size: 0,
                    created_unix_microseconds: 0,
                    content_text: None,
                    content_base64: None,
                    headers: Vec::new(),
                    summary: format!(
                        "Page present but message {} is absent from it (gap/missing on the persistence side) for topic '{}'.",
                        model.message_id, model.topic_id
                    ),
                },
            },
        };

        Ok(response)
    }
}
