use my_http_server::macros::{MyHttpInput, MyHttpObjectStructure};
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(Debug, MyHttpInput)]
pub struct SetMcpWritesInputContract {
    #[http_query(
        name = "enabled";
        description = "true opens the write window for 10 minutes (pressing it again while open adds another 10), false closes it immediately"
    )]
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, MyHttpObjectStructure)]
pub struct McpWritesContract {
    pub enabled: bool,
    /// Zero while the write tools are disabled.
    #[serde(rename = "remainingSecs")]
    pub remaining_secs: u64,
}

impl McpWritesContract {
    pub fn new(app: &AppContext) -> Self {
        let remaining_secs = app.mcp_writes_remaining_secs();

        Self {
            enabled: remaining_secs.is_some(),
            remaining_secs: remaining_secs.unwrap_or(0),
        }
    }
}
