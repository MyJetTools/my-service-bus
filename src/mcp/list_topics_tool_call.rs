use std::sync::Arc;

use mcp_server_middleware::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct ListTopicsInput {
    #[property(
        description = "Namespace to list topics of. Optional, absent means every namespace of this node"
    )]
    pub namespace: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct TopicSummary {
    #[property(description = "Namespace the topic belongs to")]
    pub namespace: String,
    #[property(description = "Topic id. Unique only inside its namespace")]
    pub id: String,
    #[property(description = "Next message id to be assigned on publish")]
    pub message_id: i64,
    #[property(description = "Messages published per second (last 1s)")]
    pub messages_per_second: usize,
    #[property(description = "Publish packets per second (last 1s)")]
    pub packets_per_second: usize,
    #[property(description = "Average size of a message in bytes")]
    pub avg_message_size: usize,
    #[property(description = "Total persisted size of the topic in bytes")]
    pub persist_size: usize,
    #[property(description = "Number of queues bound to this topic")]
    pub queues_count: usize,
    #[property(description = "Total number of subscribers across all queues of this topic")]
    pub subscribers_count: usize,
    #[property(description = "Number of publishers (sessions) seen recently")]
    pub publishers_count: usize,
    #[property(description = "true when the topic is configured to persist messages to disk")]
    pub persist: bool,
    #[property(
        description = "Soft-delete marker. 0 means alive; otherwise it's a unix-ms timestamp of when delete was requested"
    )]
    pub deleted: i64,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct ListTopicsResponse {
    #[property(description = "All topics with summary statistics")]
    pub topics: Vec<TopicSummary>,
}

pub struct ListTopicsHandler {
    app: Arc<AppContext>,
}

impl ListTopicsHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for ListTopicsHandler {
    const FUNC_NAME: &'static str = "mysb_list_topics";
    const DESCRIPTION: &'static str =
        "Returns every topic registered in the broker with summary statistics (message rate, sizes, counts of queues/subscribers/publishers).";
}

#[async_trait::async_trait]
impl McpToolCall<ListTopicsInput, ListTopicsResponse> for ListTopicsHandler {
    async fn execute_tool_call(
        &self,
        model: ListTopicsInput,
    ) -> Result<ListTopicsResponse, String> {
        // No namespace named means every namespace: a diagnostic listing that
        // quietly showed one namespace out of three would read as the whole node.
        let namespaces = match model.namespace.as_deref().filter(|itm| !itm.is_empty()) {
            Some(name) => {
                let namespace = self
                    .app
                    .namespaces
                    .get(name)
                    .ok_or_else(|| format!("Namespace '{}' not found", name))?;
                vec![namespace]
            }
            None => self.app.namespaces.get_all().as_ref().clone(),
        };

        let mut result = Vec::new();

        for namespace in namespaces.iter() {
            let namespace_name = namespace.name.to_string();

            for topic in namespace.topic_list.get_all().iter() {
                let summary = topic.get_topic_info(|inner| {
                    let mut subscribers_count = 0usize;
                    let mut queues_count = 0usize;
                    for queue in inner.queues.get_all() {
                        queues_count += 1;
                        subscribers_count += queue.subscribers.get_amount();
                    }

                    TopicSummary {
                        namespace: namespace_name.clone(),
                        id: inner.topic_id.to_string(),
                        message_id: inner.message_id.into(),
                        messages_per_second: inner.statistics.messages_per_second,
                        packets_per_second: inner.statistics.packets_per_second,
                        avg_message_size: inner.statistics.size_metrics.avg_message_size,
                        persist_size: inner.statistics.size_metrics.persist_size,
                        queues_count,
                        subscribers_count,
                        publishers_count: inner.publishers.len(),
                        persist: inner.persist,
                        deleted: inner.deleted,
                    }
                });

                result.push(summary);
            }
        }

        Ok(ListTopicsResponse { topics: result })
    }
}
