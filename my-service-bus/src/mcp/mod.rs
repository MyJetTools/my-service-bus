use std::sync::Arc;

use mcp_server_middleware::McpMiddleware;

use crate::app::{AppContext, APP_VERSION};

mod get_message_tool_call;
mod get_overview_tool_call;
mod get_topic_tool_call;
mod list_sessions_tool_call;
mod list_topics_tool_call;

pub use get_message_tool_call::*;
pub use get_overview_tool_call::*;
pub use get_topic_tool_call::*;
pub use list_sessions_tool_call::*;
pub use list_topics_tool_call::*;

pub fn build_middleware(app: Arc<AppContext>) -> McpMiddleware {
    let mut mcp = McpMiddleware::new(
        "/mcp",
        "my-service-bus",
        APP_VERSION,
        "MyServiceBus read-only stats: topics, queues, subscribers, sessions",
    );

    mcp.register_tool_call(Arc::new(GetOverviewHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(ListTopicsHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetTopicHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(ListSessionsHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetMessageHandler::new(app.clone())));

    mcp
}
