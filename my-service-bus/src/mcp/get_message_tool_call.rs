use std::sync::Arc;

use mcp_server_middleware::*;
use rust_extensions::base64::IntoBase64;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetMessageInput {
    #[property(description = "Topic id the message belongs to")]
    pub topic_id: String,
    #[property(description = "Message id to fetch from the persistence service")]
    pub message_id: i64,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct MessageHeaderView {
    #[property(description = "Header name")]
    pub name: String,
    #[property(description = "Header value")]
    pub text: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetMessageResponse {
    #[property(description = "Topic id the message belongs to")]
    pub topic_id: String,
    #[property(description = "Message id")]
    pub message_id: i64,
    #[property(description = "Creation time as unix microseconds")]
    pub created_unix_microseconds: i64,
    #[property(description = "Message payload size in bytes")]
    pub size: usize,
    #[property(
        description = "Message payload decoded as UTF-8 string. Null when the payload is not valid UTF-8"
    )]
    pub content_text: Option<String>,
    #[property(description = "Message payload encoded as Base64")]
    pub content_base64: String,
    #[property(description = "Message headers attached to the message")]
    pub headers: Vec<MessageHeaderView>,
}

pub struct GetMessageHandler {
    app: Arc<AppContext>,
}

impl GetMessageHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for GetMessageHandler {
    const FUNC_NAME: &'static str = "mysb_get_message";
    const DESCRIPTION: &'static str =
        "Fetches a single message by topic id and message id from the persistence service via gRPC. Returns the payload (UTF-8 when valid, otherwise Base64), headers, and creation time.";
}

#[async_trait::async_trait]
impl McpToolCall<GetMessageInput, GetMessageResponse> for GetMessageHandler {
    async fn execute_tool_call(
        &self,
        model: GetMessageInput,
    ) -> Result<GetMessageResponse, String> {
        let message = self
            .app
            .persistence_client
            .get_message(&model.topic_id, model.message_id.into())
            .await
            .map_err(|err| {
                format!(
                    "Failed to read message {} from topic '{}': {:?}",
                    model.message_id, model.topic_id, err
                )
            })?;

        let message = message.ok_or_else(|| {
            format!(
                "Message {} not found in topic '{}'",
                model.message_id, model.topic_id
            )
        })?;

        let size = message.data.len();
        let content_text = std::str::from_utf8(&message.data)
            .ok()
            .map(|s| s.to_string());
        let content_base64 = message.data.as_slice().into_base64();

        let headers = message
            .meta_data
            .into_iter()
            .map(|item| MessageHeaderView {
                name: item.key,
                text: item.value,
            })
            .collect();

        Ok(GetMessageResponse {
            topic_id: model.topic_id,
            message_id: message.message_id,
            created_unix_microseconds: message.created,
            size,
            content_text,
            content_base64,
            headers,
        })
    }
}
