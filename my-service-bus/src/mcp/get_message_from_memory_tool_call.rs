use std::sync::Arc;

use mcp_server_middleware::*;
use my_service_bus::abstractions::MessageId;
use my_service_bus::shared::sub_page::SubPageId;
use rust_extensions::base64::IntoBase64;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;
use crate::sub_page::GetMessageResult;

use super::MessageHeaderView;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetMessageFromMemoryInput {
    #[property(description = "Topic id the message belongs to")]
    pub topic_id: String,
    #[property(description = "Message id to fetch from the in-memory page cache")]
    pub message_id: i64,
    #[property(description = "Namespace to work in. Optional, absent means the default namespace")]
    pub namespace: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetMessageFromMemoryResponse {
    #[property(description = "Topic id")]
    pub topic_id: String,
    #[property(description = "Message id")]
    pub message_id: i64,
    #[property(description = "Sub-page id this message id maps to")]
    pub sub_page_id: i64,
    #[property(
        description = "One of: 'loaded' (payload present), 'missing' (placeholder, no payload on persistence), 'not_loaded' (sub-page is in memory but this slot is absent), 'sub_page_not_loaded' (the whole sub-page is not in memory)"
    )]
    pub state: String,
    #[property(description = "true only when the payload is present in memory (state == 'loaded')")]
    pub found: bool,
    #[property(description = "Creation time as unix microseconds (0 when not loaded)")]
    pub created_unix_microseconds: i64,
    #[property(description = "Payload size in bytes (0 when not loaded)")]
    pub size: usize,
    #[property(description = "true when the message is still awaiting persistence confirmation")]
    pub pending_persist: bool,
    #[property(
        description = "Payload decoded as UTF-8. Null when not loaded or when the payload is not valid UTF-8"
    )]
    pub content_text: Option<String>,
    #[property(description = "Payload encoded as Base64. Null when not loaded")]
    pub content_base64: Option<String>,
    #[property(description = "Message headers (empty when not loaded)")]
    pub headers: Vec<MessageHeaderView>,
}

pub struct GetMessageFromMemoryHandler {
    app: Arc<AppContext>,
}

impl GetMessageFromMemoryHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for GetMessageFromMemoryHandler {
    const FUNC_NAME: &'static str = "mysb_get_message_from_memory";
    const DESCRIPTION: &'static str =
        "Fetches a single message payload from the in-memory page cache (not the persistence service). Works for non-persisted topics too. Returns the payload (UTF-8 when valid, plus Base64), headers and creation time, or a state explaining why it is not available.";
}

#[async_trait::async_trait]
impl McpToolCall<GetMessageFromMemoryInput, GetMessageFromMemoryResponse> for GetMessageFromMemoryHandler {
    async fn execute_tool_call(
        &self,
        model: GetMessageFromMemoryInput,
    ) -> Result<GetMessageFromMemoryResponse, String> {
        let namespace = self
            .app
            .namespaces
            .get_or_create_optional(model.namespace.as_deref())
            .map_err(|err| format!("Invalid namespace. {}", err))?;

        let topic = namespace
            .topic_list
            .get(&model.topic_id)
            .ok_or_else(|| format!("Topic '{}' not found", model.topic_id))?;

        let message_id: MessageId = model.message_id.into();
        let sub_page_id: SubPageId = message_id.into();

        let response = topic.get_topic_info(|inner| {
            let topic_id = inner.topic_id.to_string();
            let sub_page_id_value = sub_page_id.get_value();

            let sub_page = match inner.pages.get_sub_page(sub_page_id) {
                Some(sub_page) => sub_page,
                None => {
                    return GetMessageFromMemoryResponse {
                        topic_id,
                        message_id: model.message_id,
                        sub_page_id: sub_page_id_value,
                        state: "sub_page_not_loaded".to_string(),
                        found: false,
                        created_unix_microseconds: 0,
                        size: 0,
                        pending_persist: false,
                        content_text: None,
                        content_base64: None,
                        headers: Vec::new(),
                    };
                }
            };

            let pending_persist = sub_page.is_pending_persist(message_id);

            match sub_page.get_message(message_id) {
                GetMessageResult::Message(content) => {
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

                    GetMessageFromMemoryResponse {
                        topic_id,
                        message_id: model.message_id,
                        sub_page_id: sub_page_id_value,
                        state: "loaded".to_string(),
                        found: true,
                        created_unix_microseconds: content.time.unix_microseconds,
                        size,
                        pending_persist,
                        content_text,
                        content_base64: Some(content_base64),
                        headers,
                    }
                }
                GetMessageResult::Missing => GetMessageFromMemoryResponse {
                    topic_id,
                    message_id: model.message_id,
                    sub_page_id: sub_page_id_value,
                    state: "missing".to_string(),
                    found: false,
                    created_unix_microseconds: 0,
                    size: 0,
                    pending_persist,
                    content_text: None,
                    content_base64: None,
                    headers: Vec::new(),
                },
                GetMessageResult::NotLoaded => GetMessageFromMemoryResponse {
                    topic_id,
                    message_id: model.message_id,
                    sub_page_id: sub_page_id_value,
                    state: "not_loaded".to_string(),
                    found: false,
                    created_unix_microseconds: 0,
                    size: 0,
                    pending_persist,
                    content_text: None,
                    content_base64: None,
                    headers: Vec::new(),
                },
            }
        });

        Ok(response)
    }
}
