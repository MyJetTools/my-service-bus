use std::sync::Arc;

use mcp_server_middleware::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetDebugConsoleInput {
    #[property(description = "Return only the last N records (optional)")]
    pub tail: Option<i64>,
    #[property(description = "Clear the buffer after reading (optional, default false)")]
    pub clear: Option<bool>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct DebugConsoleRecordView {
    #[property(description = "Event date (RFC3339)")]
    pub date: String,
    #[property(description = "Event data")]
    pub data: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetDebugConsoleResponse {
    #[property(description = "true when the debug console is currently tracing a target")]
    pub enabled: bool,
    #[property(description = "Topic currently traced; null when disabled")]
    pub topic_id: Option<String>,
    #[property(description = "Queue currently traced; null = all queues of the topic (or disabled)")]
    pub queue_id: Option<String>,
    #[property(description = "Number of records returned")]
    pub records_count: usize,
    #[property(description = "Debug records, oldest first, each with event date and data")]
    pub records: Vec<DebugConsoleRecordView>,
}

pub struct GetDebugConsoleHandler {
    app: Arc<AppContext>,
}

impl GetDebugConsoleHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for GetDebugConsoleHandler {
    const FUNC_NAME: &'static str = "mysb_get_debug_console";
    const DESCRIPTION: &'static str =
        "Returns the in-memory debug console records (each with event date and data, up to 1000 entries). What gets traced is selected at runtime via the POST /api/Debug/Console/Target HTTP endpoint (topic + optional queue). Use 'tail' to limit how many records to return and 'clear' to reset the buffer after reading.";
}

#[async_trait::async_trait]
impl McpToolCall<GetDebugConsoleInput, GetDebugConsoleResponse> for GetDebugConsoleHandler {
    async fn execute_tool_call(
        &self,
        model: GetDebugConsoleInput,
    ) -> Result<GetDebugConsoleResponse, String> {
        let tail = model
            .tail
            .and_then(|v| if v > 0 { Some(v as usize) } else { None });

        let records = self.app.debug_console.get_records(tail);
        let target = self.app.debug_console.get_target();

        if let Some(true) = model.clear {
            self.app.debug_console.clear();
        }

        Ok(GetDebugConsoleResponse {
            enabled: target.is_some(),
            topic_id: target.as_ref().map(|t| t.topic_id.clone()),
            queue_id: target.as_ref().and_then(|t| t.queue_id.clone()),
            records_count: records.len(),
            records: records
                .into_iter()
                .map(|r| DebugConsoleRecordView {
                    date: r.date_rfc3339(),
                    data: r.message,
                })
                .collect(),
        })
    }
}
