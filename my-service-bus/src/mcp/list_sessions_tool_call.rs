use std::sync::Arc;

use mcp_server_middleware::*;
use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct ListSessionsInput {}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct SessionView {
    #[property(description = "Session id")]
    pub id: i64,
    #[property(description = "Client-provided name (e.g. publisher/subscriber app name)")]
    pub name: String,
    #[property(description = "Connection type. Includes TCP protocol version when applicable")]
    pub session_type: String,
    #[property(description = "Remote endpoint IP")]
    pub ip: String,
    #[property(description = "Client-reported library/app version, when provided")]
    pub version: Option<String>,
    #[property(description = "Client-reported environment info, when provided")]
    pub env_info: Option<String>,
    #[property(description = "Time since the session was established (e.g. \"00:00:05\")")]
    pub connected: String,
    #[property(description = "Time since the last incoming packet from the client")]
    pub last_incoming: String,
    #[property(description = "Total bytes read from this session")]
    pub read_size: usize,
    #[property(description = "Total bytes written to this session")]
    pub written_size: usize,
    #[property(description = "Bytes read per second over the last 1s")]
    pub read_per_sec: usize,
    #[property(description = "Bytes written per second over the last 1s")]
    pub written_per_sec: usize,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct ListSessionsResponse {
    #[property(description = "All currently connected sessions")]
    pub sessions: Vec<SessionView>,
}

pub struct ListSessionsHandler {
    app: Arc<AppContext>,
}

impl ListSessionsHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for ListSessionsHandler {
    const FUNC_NAME: &'static str = "mysb_list_sessions";
    const DESCRIPTION: &'static str =
        "Returns every currently connected session (publishers and subscribers) with traffic metrics.";
}

#[async_trait::async_trait]
impl McpToolCall<ListSessionsInput, ListSessionsResponse> for ListSessionsHandler {
    async fn execute_tool_call(
        &self,
        _model: ListSessionsInput,
    ) -> Result<ListSessionsResponse, String> {
        let (_, all_sessions) = self.app.sessions.get_snapshot();
        let now = DateTimeAsMicroseconds::now();

        let mut sessions = Vec::with_capacity(all_sessions.len());

        for session in &all_sessions {
            let metrics = session.get_metrics();
            let session_type = if let Some(prot_ver) = metrics.tcp_protocol_version {
                format!("{}[{}]", session.get_type_as_str(), prot_ver)
            } else {
                session.get_type_as_str().to_string()
            };
            let name_and_version = session.get_name_and_version();

            sessions.push(SessionView {
                id: session.session_id.get_value(),
                ip: metrics.ip,
                session_type,
                name: name_and_version.name,
                version: name_and_version.version,
                env_info: name_and_version.env_info,
                connected: rust_extensions::duration_utils::duration_to_string(
                    now.duration_since(metrics.connected).as_positive_or_zero(),
                ),
                last_incoming: rust_extensions::duration_utils::duration_to_string(
                    now.duration_since(metrics.connection_metrics.last_incoming_moment)
                        .as_positive_or_zero(),
                ),
                read_size: metrics.connection_metrics.read,
                written_size: metrics.connection_metrics.written,
                read_per_sec: metrics.connection_metrics.read_per_sec,
                written_per_sec: metrics.connection_metrics.written_per_sec,
            });
        }

        Ok(ListSessionsResponse { sessions })
    }
}
