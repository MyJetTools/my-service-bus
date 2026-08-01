use std::sync::Arc;

use mcp_server_middleware::*;
use serde::{Deserialize, Serialize};

use crate::app::{AppContext, APP_VERSION};

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetOverviewInput {}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetOverviewResponse {
    #[property(description = "Number of namespaces this node holds")]
    pub namespaces_count: usize,
    #[property(description = "Number of topics registered in the broker, across every namespace")]
    pub topics_count: usize,
    #[property(description = "Total number of queues across all topics")]
    pub queues_count: usize,
    #[property(description = "Total number of subscribers across all queues")]
    pub subscribers_count: usize,
    #[property(description = "Total number of active TCP/UnixSocket sessions")]
    pub sessions_count: usize,
    #[property(description = "Used system memory in bytes")]
    pub used_memory: u64,
    #[property(description = "Total system memory in bytes")]
    pub total_memory: u64,
    #[property(description = "Persistence service version reported by the persistence grpc client")]
    pub persistence_version: String,
    #[property(description = "MyServiceBus main-node application version")]
    pub app_version: String,
}

pub struct GetOverviewHandler {
    app: Arc<AppContext>,
}

impl GetOverviewHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for GetOverviewHandler {
    const FUNC_NAME: &'static str = "mysb_get_overview";
    const DESCRIPTION: &'static str =
        "High-level snapshot of the MyServiceBus node: counts of topics, queues, subscribers, sessions and memory usage.";
}

#[async_trait::async_trait]
impl McpToolCall<GetOverviewInput, GetOverviewResponse> for GetOverviewHandler {
    async fn execute_tool_call(
        &self,
        _model: GetOverviewInput,
    ) -> Result<GetOverviewResponse, String> {
        let namespaces = self.app.namespaces.get_all();

        let mut topics_count = 0usize;
        let mut queues_count = 0usize;
        let mut subscribers_count = 0usize;

        for namespace in namespaces.iter() {
            let topics = namespace.topic_list.get_all();
            topics_count += topics.len();

            for topic in topics.iter() {
                let (q, s) = topic.get_topic_info(|inner| {
                    let mut q = 0usize;
                    let mut s = 0usize;
                    for queue in inner.queues.get_all() {
                        q += 1;
                        s += queue.subscribers.get_amount();
                    }
                    (q, s)
                });
                queues_count += q;
                subscribers_count += s;
            }
        }

        let (_, sessions) = self.app.sessions.get_snapshot();

        let mut sys_info = sysinfo::System::new_all();
        sys_info.refresh_all();

        Ok(GetOverviewResponse {
            namespaces_count: namespaces.len(),
            topics_count,
            queues_count,
            subscribers_count,
            sessions_count: sessions.len(),
            used_memory: sys_info.used_memory(),
            total_memory: sys_info.total_memory(),
            persistence_version: self.app.persistence_version.get(),
            app_version: APP_VERSION.to_string(),
        })
    }
}
