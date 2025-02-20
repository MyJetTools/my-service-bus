use std::sync::Arc;

use crate::{app::AppContext, sessions::MyServiceBusSession};

use rust_extensions::date_time::DateTimeAsMicroseconds;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionJsonResult {
    pub id: i64,
    pub name: String,
    #[serde(rename = "type")]
    pub session_type: String,
    pub ip: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(rename = "envInfo")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_info: Option<String>,
    pub connected: String,
    #[serde(rename = "lastIncoming")]
    pub last_incoming: String,
    #[serde(rename = "readSize")]
    pub read_size: usize,
    #[serde(rename = "writtenSize")]
    pub written_size: usize,
    #[serde(rename = "readPerSec")]
    pub read_per_sec: usize,
    #[serde(rename = "writtenPerSec")]
    pub written_per_sec: usize,
}

impl SessionJsonResult {
    pub async fn new(session: &Arc<dyn MyServiceBusSession + Send + Sync + 'static>) -> Self {
        let now = DateTimeAsMicroseconds::now();

        let session_metrics = session.get_metrics();

        let session_type = if let Some(prot_ver) = session_metrics.tcp_protocol_version {
            format!("{}[{}]", session.get_session_type().as_str(), prot_ver)
        } else {
            session.get_session_type().as_str().to_string()
        };

        let name_and_version = session.get_name_and_version();

        Self {
            id: session.get_session_id().get_value(),
            ip: session_metrics.ip,
            session_type,
            name: name_and_version.name,
            version: name_and_version.version,
            env_info: name_and_version.env_info,
            connected: rust_extensions::duration_utils::duration_to_string(
                now.duration_since(session_metrics.connected)
                    .as_positive_or_zero(),
            ),
            last_incoming: rust_extensions::duration_utils::duration_to_string(
                now.duration_since(session_metrics.connection_metrics.last_incoming_moment)
                    .as_positive_or_zero(),
            ),
            read_size: session_metrics.connection_metrics.read,
            written_size: session_metrics.connection_metrics.written,
            read_per_sec: session_metrics.connection_metrics.read_per_sec,
            written_per_sec: session_metrics.connection_metrics.written_per_sec,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionsJsonResult {
    #[serde(rename = "snapshotId")]
    pub snapshot_id: usize,
    pub items: Vec<SessionJsonResult>,
}

impl SessionsJsonResult {
    pub async fn new(app: &AppContext) -> Self {
        let (sessions_snapshot_id, all_sessions) = app.sessions.get_snapshot().await;

        let mut result = SessionsJsonResult {
            snapshot_id: sessions_snapshot_id,
            items: Vec::new(),
        };

        for session in &all_sessions {
            let session_json_model = SessionJsonResult::new(session).await;
            result.items.push(session_json_model);
        }

        result
    }
}
