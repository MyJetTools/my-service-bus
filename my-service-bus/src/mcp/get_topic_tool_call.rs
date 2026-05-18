use std::sync::Arc;

use mcp_server_middleware::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetTopicInput {
    #[property(description = "Topic id to fetch details for")]
    pub topic_id: String,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct TopicQueueView {
    #[property(description = "Queue id")]
    pub id: String,
    #[property(
        description = "Queue type: 0=Permanent, 1=DeleteOnDisconnect, 2=PermanentWithSingleConnection"
    )]
    pub queue_type: u8,
    #[property(description = "Total messages currently waiting in the queue")]
    pub size: usize,
    #[property(description = "Messages currently sent to subscribers and awaiting confirmation")]
    pub on_delivery: usize,
    #[property(description = "Number of active subscribers attached to this queue")]
    pub subscribers_count: usize,
    #[property(description = "Details of each subscriber attached to this queue")]
    pub subscribers: Vec<QueueSubscriberView>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct QueueSubscriberView {
    #[property(description = "Subscriber id")]
    pub subscriber_id: i64,
    #[property(description = "Session id this subscriber belongs to")]
    pub session_id: i64,
    #[property(description = "Numeric delivery state")]
    pub delivery_state: u8,
    #[property(description = "Delivery state as a human-readable string")]
    pub delivery_state_str: String,
    #[property(description = "Number of messages currently on delivery for this subscriber")]
    pub on_delivery: usize,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct TopicPublisherView {
    #[property(description = "Session id of the publisher")]
    pub session_id: i64,
    #[property(
        description = "Activity badge counter. >0 means the publisher published in the last few seconds"
    )]
    pub active: u8,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct GetTopicResponse {
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
    #[property(description = "true when the topic is configured to persist messages to disk")]
    pub persist: bool,
    #[property(
        description = "Soft-delete marker. 0 means alive; otherwise it's a unix-ms timestamp of when delete was requested"
    )]
    pub deleted: i64,
    #[property(description = "Recent per-second messages-per-second history (newest last)")]
    pub publish_history: Vec<i32>,
    #[property(description = "Active publishers")]
    pub publishers: Vec<TopicPublisherView>,
    #[property(description = "Queues bound to the topic")]
    pub queues: Vec<TopicQueueView>,
}

pub struct GetTopicHandler {
    app: Arc<AppContext>,
}

impl GetTopicHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for GetTopicHandler {
    const FUNC_NAME: &'static str = "mysb_get_topic";
    const DESCRIPTION: &'static str =
        "Detailed view of a single topic: stats, queues with sizes/on-delivery, every subscriber and every publisher.";
}

#[async_trait::async_trait]
impl McpToolCall<GetTopicInput, GetTopicResponse> for GetTopicHandler {
    async fn execute_tool_call(
        &self,
        model: GetTopicInput,
    ) -> Result<GetTopicResponse, String> {
        let topic = self
            .app
            .topic_list
            .get(&model.topic_id)
            .ok_or_else(|| format!("Topic '{}' not found", model.topic_id))?;

        let response = topic.get_topic_info(|inner| {
            let publishers = inner
                .publishers
                .iter()
                .map(|(session_id, active)| TopicPublisherView {
                    session_id: session_id.get_value(),
                    active: *active,
                })
                .collect();

            let mut queues = Vec::new();
            for queue in inner.queues.get_all() {
                let subscribers: Vec<QueueSubscriberView> = match queue.subscribers.get_all() {
                    Some(list) => list
                        .into_iter()
                        .map(|s| QueueSubscriberView {
                            subscriber_id: s.id.get_value(),
                            session_id: s.session.session_id.get_value(),
                            delivery_state: s.delivery_state.to_u8(),
                            delivery_state_str: s.delivery_state.as_str().to_string(),
                            on_delivery: s.get_on_delivery_amount(),
                        })
                        .collect(),
                    None => Vec::new(),
                };

                queues.push(TopicQueueView {
                    id: queue.queue_id.to_string(),
                    queue_type: queue.queue_type.into_u8(),
                    size: queue.get_queue_size(),
                    on_delivery: queue.get_on_delivery(),
                    subscribers_count: queue.subscribers.get_amount(),
                    subscribers,
                });
            }

            GetTopicResponse {
                id: inner.topic_id.to_string(),
                message_id: inner.message_id.into(),
                messages_per_second: inner.statistics.messages_per_second,
                packets_per_second: inner.statistics.packets_per_second,
                avg_message_size: inner.statistics.size_metrics.avg_message_size,
                persist_size: inner.statistics.size_metrics.persist_size,
                persist: inner.persist,
                deleted: inner.deleted,
                publish_history: inner.statistics.publish_history.get(),
                publishers,
                queues,
            }
        });

        Ok(response)
    }
}
