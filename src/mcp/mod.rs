use std::sync::Arc;

use mcp_server_middleware::McpMiddleware;

use crate::app::{AppContext, APP_VERSION};

mod delete_queue_tool_call;
mod delete_topic_tool_call;
mod get_debug_console_tool_call;
mod get_message_from_memory_tool_call;
mod get_message_tool_call;
mod get_overview_tool_call;
mod get_page_messages_tool_call;
mod get_topic_pages_tool_call;
mod get_topic_tool_call;
mod list_sessions_tool_call;
mod list_topics_tool_call;
mod persistence_get_message_tool_call;
mod persistence_load_page_tool_call;
mod set_topic_persist_tool_call;

mod write_gate;

pub use delete_queue_tool_call::*;
pub use delete_topic_tool_call::*;
pub use get_debug_console_tool_call::*;
pub use get_message_from_memory_tool_call::*;
pub use get_message_tool_call::*;
pub use get_overview_tool_call::*;
pub use get_page_messages_tool_call::*;
pub use get_topic_pages_tool_call::*;
pub use get_topic_tool_call::*;
pub use list_sessions_tool_call::*;
pub use list_topics_tool_call::*;
pub use persistence_get_message_tool_call::*;
pub use persistence_load_page_tool_call::*;
pub use set_topic_persist_tool_call::*;

pub fn build_middleware(app: Arc<AppContext>) -> McpMiddleware {
    let mut mcp = McpMiddleware::new(
        "/mcp",
        "my-service-bus",
        APP_VERSION,
        "MyServiceBus stats: topics, queues, subscribers, sessions, in-memory pages and messages. Write tools (set topic persist, delete queue, delete topic) are refused unless a human has enabled MCP writes in the UI.",
    );

    mcp.register_tool_call(Arc::new(GetOverviewHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(ListTopicsHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetTopicHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetTopicPagesHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetPageMessagesHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(ListSessionsHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetMessageHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetMessageFromMemoryHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(PersistenceLoadPageHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(PersistenceGetMessageHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(GetDebugConsoleHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(SetTopicPersistHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(DeleteQueueHandler::new(app.clone())));
    mcp.register_tool_call(Arc::new(DeleteTopicHandler::new(app.clone())));

    mcp
}
