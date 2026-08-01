use std::sync::Arc;

use mcp_server_middleware::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct DeleteQueueInput {
    #[property(description = "Topic id the queue belongs to")]
    pub topic_id: String,
    #[property(description = "Queue id to delete")]
    pub queue_id: String,
    #[property(description = "Namespace to work in. Optional, absent means the default namespace")]
    pub namespace: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct DeleteQueueResponse {
    #[property(description = "Namespace the queue was deleted from")]
    pub namespace: String,
    #[property(description = "Topic id")]
    pub topic_id: String,
    #[property(description = "Queue id that was deleted")]
    pub queue_id: String,
    #[property(description = "How many messages the queue still held when it was deleted")]
    pub messages_left: i64,
    #[property(description = "How many subscribers were attached when it was deleted")]
    pub subscribers_left: usize,
}

pub struct DeleteQueueHandler {
    app: Arc<AppContext>,
}

impl DeleteQueueHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for DeleteQueueHandler {
    const FUNC_NAME: &'static str = "mysb_delete_queue";
    const DESCRIPTION: &'static str =
        "Deletes a queue from a topic. This is a DESTRUCTIVE WRITE operation and it is IRREVERSIBLE: the queue's delivery cursor is gone, so messages it had not consumed yet will never be delivered to it. The queue is deleted even when it still holds messages or has live subscribers - the response reports both counts as they were at the moment of deletion, so check them first if that matters. Requires MCP writes to be enabled by a human in the UI.";
}

#[async_trait::async_trait]
impl McpToolCall<DeleteQueueInput, DeleteQueueResponse> for DeleteQueueHandler {
    async fn execute_tool_call(
        &self,
        model: DeleteQueueInput,
    ) -> Result<DeleteQueueResponse, String> {
        super::write_gate::ensure_mcp_writes_enabled(self.app.as_ref())?;

        let namespace = self
            .app
            .namespaces
            .get_or_create_optional(model.namespace.as_deref())
            .map_err(|err| format!("Invalid namespace. {}", err))?;

        let topic = namespace
            .topic_list
            .get(&model.topic_id)
            .ok_or_else(|| format!("Topic '{}' not found", model.topic_id))?;

        // Read the queue's state before the delete, so the answer says what was
        // actually thrown away rather than just "ok".
        let (messages_left, subscribers_left) = topic
            .get_topic_info(|inner| {
                inner
                    .queues
                    .get(model.queue_id.as_str())
                    .map(|queue| (queue.get_queue_size() as i64, queue.subscribers.get_amount()))
            })
            .ok_or_else(|| {
                format!(
                    "Queue '{}' not found in topic '{}'",
                    model.queue_id, model.topic_id
                )
            })?;

        crate::operations::queues::delete_queue(
            self.app.as_ref(),
            &namespace,
            model.topic_id.as_str(),
            model.queue_id.as_str(),
        )
        .await
        .map_err(|err| {
            format!(
                "Failed to delete queue '{}' of topic '{}': {:?}",
                model.queue_id, model.topic_id, err
            )
        })?;

        Ok(DeleteQueueResponse {
            namespace: namespace.name.to_string(),
            topic_id: model.topic_id,
            queue_id: model.queue_id,
            messages_left,
            subscribers_left,
        })
    }
}
