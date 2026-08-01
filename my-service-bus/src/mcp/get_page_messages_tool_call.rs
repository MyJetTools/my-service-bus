use std::sync::Arc;

use mcp_server_middleware::*;
use my_service_bus::shared::sub_page::SubPageId;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetPageMessagesInput {
    #[property(description = "Topic id the sub-page belongs to")]
    pub topic_id: String,
    #[property(
        description = "Sub-page id to inspect (as reported by mysb_get_topic_pages). Holds up to 1000 message ids"
    )]
    pub sub_page_id: i64,
    #[property(description = "Namespace to work in. Optional, absent means the default namespace")]
    pub namespace: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct PageMessageView {
    #[property(description = "Message id")]
    pub message_id: i64,
    #[property(description = "true when the payload is loaded in memory; false for a missing placeholder")]
    pub loaded: bool,
    #[property(description = "Payload size in bytes (0 for a missing placeholder)")]
    pub size: usize,
    #[property(description = "Creation time as unix microseconds (0 for a missing placeholder)")]
    pub created_unix_microseconds: i64,
    #[property(description = "true when the message is still awaiting persistence confirmation")]
    pub pending_persist: bool,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetPageMessagesResponse {
    #[property(description = "Topic id")]
    pub topic_id: String,
    #[property(description = "Sub-page id that was inspected")]
    pub sub_page_id: i64,
    #[property(description = "First message id this sub-page can hold")]
    pub first_message_id: i64,
    #[property(description = "Last message id this sub-page can hold")]
    pub last_message_id: i64,
    #[property(description = "true when this sub-page is currently held in memory")]
    pub loaded: bool,
    #[property(description = "Number of message slots returned")]
    pub total_messages: usize,
    #[property(description = "Every message slot held in this sub-page, ordered by message id")]
    pub messages: Vec<PageMessageView>,
}

pub struct GetPageMessagesHandler {
    app: Arc<AppContext>,
}

impl GetPageMessagesHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for GetPageMessagesHandler {
    const FUNC_NAME: &'static str = "mysb_get_page_messages";
    const DESCRIPTION: &'static str =
        "Lists the messages currently held in memory inside a single sub-page (metadata only: id, loaded/missing, size, created time, pending-persist). Use mysb_get_message_from_memory to fetch a payload. If the sub-page is not in memory, loaded=false and the message list is empty.";
}

#[async_trait::async_trait]
impl McpToolCall<GetPageMessagesInput, GetPageMessagesResponse> for GetPageMessagesHandler {
    async fn execute_tool_call(
        &self,
        model: GetPageMessagesInput,
    ) -> Result<GetPageMessagesResponse, String> {
        let namespace = self
            .app
            .namespaces
            .get_or_create_optional(model.namespace.as_deref())
            .map_err(|err| format!("Invalid namespace. {}", err))?;

        let topic = namespace
            .topic_list
            .get(&model.topic_id)
            .ok_or_else(|| format!("Topic '{}' not found", model.topic_id))?;

        let sub_page_id = SubPageId::new(model.sub_page_id);

        let response = topic.get_topic_info(|inner| {
            let first_message_id = sub_page_id.get_first_message_id().get_value();
            let last_message_id = sub_page_id.get_last_message_id().get_value();

            match inner.pages.get_sub_page(sub_page_id) {
                Some(sub_page) => {
                    let messages: Vec<PageMessageView> = sub_page
                        .get_messages_meta()
                        .into_iter()
                        .map(|m| PageMessageView {
                            message_id: m.message_id,
                            loaded: m.loaded,
                            size: m.size,
                            created_unix_microseconds: m.created_unix_microseconds,
                            pending_persist: m.pending_persist,
                        })
                        .collect();

                    GetPageMessagesResponse {
                        topic_id: inner.topic_id.to_string(),
                        sub_page_id: sub_page_id.get_value(),
                        first_message_id,
                        last_message_id,
                        loaded: true,
                        total_messages: messages.len(),
                        messages,
                    }
                }
                None => GetPageMessagesResponse {
                    topic_id: inner.topic_id.to_string(),
                    sub_page_id: sub_page_id.get_value(),
                    first_message_id,
                    last_message_id,
                    loaded: false,
                    total_messages: 0,
                    messages: Vec::new(),
                },
            }
        });

        Ok(response)
    }
}
