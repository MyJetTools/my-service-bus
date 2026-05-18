use std::sync::Arc;

use mcp_server_middleware::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct ListTopicsInput {}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct TopicSummary {
    #[property(description = "Topic id")]
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
        _model: ListTopicsInput,
    ) -> Result<ListTopicsResponse, String> {
        let topics = self.app.topic_list.get_all();
        let mut result = Vec::with_capacity(topics.len());

        for topic in topics.iter() {
            let summary = topic.get_topic_info(|inner| {
                let mut subscribers_count = 0usize;
                let mut queues_count = 0usize;
                for queue in inner.queues.get_all() {
                    queues_count += 1;
                    subscribers_count += queue.subscribers.get_amount();
                }

                TopicSummary {
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

        Ok(ListTopicsResponse { topics: result })
    }
}
