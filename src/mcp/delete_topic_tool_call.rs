use std::{sync::Arc, time::Duration};

use mcp_server_middleware::*;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

/// How long a deleted topic stays recoverable when the caller does not say.
/// Deleting is a soft delete first — the GC only wipes the data once this has
/// passed — so the default leaves a full day to notice a mistake and restore.
const DEFAULT_HARD_DELETE_AFTER: Duration = Duration::from_secs(24 * 60 * 60);

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct DeleteTopicInput {
    #[property(description = "Topic id to delete")]
    pub topic_id: String,
    #[property(
        description = "Seconds to keep the topic recoverable before its data is wiped for good. Optional, defaults to 86400 (24h). 0 means the next GC tick wipes it - there is no way back after that"
    )]
    pub hard_delete_after_seconds: Option<u64>,
    #[property(description = "Namespace to work in. Optional, absent means the default namespace")]
    pub namespace: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct DeleteTopicResponse {
    #[property(description = "Namespace the topic was deleted in")]
    pub namespace: String,
    #[property(description = "Topic id that was marked as deleted")]
    pub topic_id: String,
    #[property(
        description = "Moment (RFC 3339) after which the GC wipes the topic's data for good. Until then mysb_restore_topic - or PUT /api/Topics/Restore - brings it back"
    )]
    pub hard_delete_moment: String,
    #[property(description = "How many queues the topic still had when it was deleted")]
    pub queues_left: usize,
    #[property(description = "How many publishers the topic still had when it was deleted")]
    pub publishers_left: usize,
}

pub struct DeleteTopicHandler {
    app: Arc<AppContext>,
}

impl DeleteTopicHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for DeleteTopicHandler {
    const FUNC_NAME: &'static str = "mysb_delete_topic";
    const DESCRIPTION: &'static str =
        "Marks a topic as deleted. This is a DESTRUCTIVE WRITE operation. It is a SOFT delete: the topic disappears from the broker immediately, and its persisted data is wiped by the GC only once hard_delete_after_seconds has passed - until then the topic can be restored. The topic is deleted even when it still has queues or publishers; the response reports both counts as they were at the moment of deletion, so check them first if that matters. Requires MCP writes to be enabled by a human in the UI.";
}

#[async_trait::async_trait]
impl McpToolCall<DeleteTopicInput, DeleteTopicResponse> for DeleteTopicHandler {
    async fn execute_tool_call(
        &self,
        model: DeleteTopicInput,
    ) -> Result<DeleteTopicResponse, String> {
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

        // Read what the topic still holds before the delete, so the answer says
        // what was actually taken out of service rather than just "ok".
        let (queues_left, publishers_left) = topic.get_topic_info(|inner| {
            (inner.queues.get_all().count(), inner.publishers.len())
        });

        let keep_for = match model.hard_delete_after_seconds {
            Some(seconds) => Duration::from_secs(seconds),
            None => DEFAULT_HARD_DELETE_AFTER,
        };

        let hard_delete_moment = DateTimeAsMicroseconds::now().add(keep_for);

        crate::operations::delete_topic(
            &self.app,
            &namespace,
            model.topic_id.as_str(),
            hard_delete_moment,
        )
        .await
        .map_err(|err| format!("Failed to delete topic '{}': {:?}", model.topic_id, err))?;

        Ok(DeleteTopicResponse {
            namespace: namespace.name.to_string(),
            topic_id: model.topic_id,
            hard_delete_moment: hard_delete_moment.to_rfc3339(),
            queues_left,
            publishers_left,
        })
    }
}
