use std::sync::Arc;

use my_http_server::macros::http_route;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::{McpWritesContract, SetMcpWritesInputContract};

#[http_route(
    method: "POST",
    route: "/api/Mcp/Writes",
    controller: "Mcp",
    description: "Enables or disables the MCP write tools (set topic persist, delete queue, delete topic). Enabling opens a 10-minute window after which they auto-disable; enabling again while it is open adds another 10 minutes. Runtime-only - a node restart always leaves MCP writes disabled.",
    summary: "Enable/disable MCP writes",
    input_data: "SetMcpWritesInputContract",
    result: [
        {status_code: 200, description: "State of the write window after the call", model: "McpWritesContract"},
    ]
)]
pub struct SetMcpWritesAction {
    app: Arc<AppContext>,
}

impl SetMcpWritesAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &SetMcpWritesAction,
    input_data: SetMcpWritesInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    if input_data.enabled {
        action.app.enable_mcp_writes();
    } else {
        action.app.disable_mcp_writes();
    }

    HttpOutput::as_json(McpWritesContract::new(action.app.as_ref()))
        .into_ok_result(false)
        .into()
}
