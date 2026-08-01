use std::sync::Arc;

use mcp_server_middleware::*;
use my_service_bus::shared::page_id::PageId;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetTopicPagesInput {
    #[property(description = "Topic id to list in-memory message pages for")]
    pub topic_id: String,
    #[property(description = "Namespace to work in. Optional, absent means the default namespace")]
    pub namespace: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct SubPageView {
    #[property(description = "Sub-page id. Each sub-page holds up to 1000 consecutive message ids")]
    pub sub_page_id: i64,
    #[property(description = "Page id (100k-message page) this sub-page belongs to")]
    pub page_id: i64,
    #[property(description = "First message id covered by this sub-page")]
    pub first_message_id: i64,
    #[property(description = "Last message id covered by this sub-page")]
    pub last_message_id: i64,
    #[property(description = "Message slots currently held in this sub-page (loaded + missing placeholders)")]
    pub messages_amount: usize,
    #[property(description = "Slots that have their payload loaded in memory")]
    pub loaded_amount: usize,
    #[property(description = "Slots that are missing placeholders (no payload on the persistence side)")]
    pub missing_amount: usize,
    #[property(description = "Total payload size held by this sub-page in bytes")]
    pub data_size: usize,
    #[property(description = "Messages in this sub-page still awaiting persistence confirmation")]
    pub pending_persist_amount: usize,
    #[property(description = "When this sub-page was last accessed, as unix microseconds")]
    pub last_accessed_unix_microseconds: i64,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetTopicPagesResponse {
    #[property(description = "Topic id")]
    pub topic_id: String,
    #[property(description = "Next message id to be assigned on publish")]
    pub current_message_id: i64,
    #[property(description = "true when the topic persists messages to disk")]
    pub persist: bool,
    #[property(description = "Number of sub-pages currently held in memory")]
    pub sub_pages_count: usize,
    #[property(description = "Every sub-page currently loaded in memory, oldest first")]
    pub sub_pages: Vec<SubPageView>,
}

pub struct GetTopicPagesHandler {
    app: Arc<AppContext>,
}

impl GetTopicPagesHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for GetTopicPagesHandler {
    const FUNC_NAME: &'static str = "mysb_get_topic_pages";
    const DESCRIPTION: &'static str =
        "Lists the message sub-pages currently held in memory for a topic, with per-sub-page metrics (id range, loaded/missing counts, size, pending-persist, last access). Reads in-memory state, not persistence.";
}

#[async_trait::async_trait]
impl McpToolCall<GetTopicPagesInput, GetTopicPagesResponse> for GetTopicPagesHandler {
    async fn execute_tool_call(
        &self,
        model: GetTopicPagesInput,
    ) -> Result<GetTopicPagesResponse, String> {
        let namespace = self
            .app
            .namespaces
            .get_or_create_optional(model.namespace.as_deref())
            .map_err(|err| format!("Invalid namespace. {}", err))?;

        let topic = namespace
            .topic_list
            .get(&model.topic_id)
            .ok_or_else(|| format!("Topic '{}' not found", model.topic_id))?;

        let response = topic.get_topic_info(|inner| {
            let mut sub_pages = Vec::new();

            for sub_page in inner.pages.sub_pages.iter() {
                let sub_page_id = sub_page.get_id();
                let page_id: PageId = sub_page_id.into();
                let metrics = sub_page.get_size_metrics();
                let (loaded_amount, missing_amount) = sub_page.get_loaded_and_missing();

                sub_pages.push(SubPageView {
                    sub_page_id: sub_page_id.get_value(),
                    page_id: page_id.get_value(),
                    first_message_id: sub_page_id.get_first_message_id().get_value(),
                    last_message_id: sub_page_id.get_last_message_id().get_value(),
                    messages_amount: metrics.messages_amount,
                    loaded_amount,
                    missing_amount,
                    data_size: metrics.data_size,
                    pending_persist_amount: metrics.persist_size,
                    last_accessed_unix_microseconds: sub_page.last_accessed.unix_microseconds,
                });
            }

            GetTopicPagesResponse {
                topic_id: inner.topic_id.to_string(),
                current_message_id: inner.message_id.into(),
                persist: inner.persist,
                sub_pages_count: sub_pages.len(),
                sub_pages,
            }
        });

        Ok(response)
    }
}
