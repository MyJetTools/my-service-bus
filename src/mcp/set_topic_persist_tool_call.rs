use std::sync::Arc;

use mcp_server_middleware::*;
use serde::{Deserialize, Serialize};

use crate::app::AppContext;

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct SetTopicPersistInput {
    #[property(description = "Topic id to configure")]
    pub topic_id: String,
    #[property(
        description = "Whether the topic should persist messages to disk. Optional, defaults to true (enable persistence)"
    )]
    pub persist: Option<bool>,
    #[property(description = "Namespace to work in. Optional, absent means the default namespace")]
    pub namespace: Option<String>,
}

#[derive(ApplyJsonSchema, Debug, Serialize, Deserialize)]
pub struct SetTopicPersistResponse {
    #[property(description = "Topic id")]
    pub topic_id: String,
    #[property(description = "Persist flag the topic had before this call")]
    pub previous_persist: bool,
    #[property(description = "Persist flag the topic has after this call")]
    pub persist: bool,
    #[property(description = "true when the call actually changed the flag (false = already configured)")]
    pub changed: bool,
}

pub struct SetTopicPersistHandler {
    app: Arc<AppContext>,
}

impl SetTopicPersistHandler {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

impl ToolDefinition for SetTopicPersistHandler {
    const FUNC_NAME: &'static str = "mysb_set_topic_persist";
    const DESCRIPTION: &'static str =
        "Configures whether a topic persists its messages to disk (writes to the persistence service). Pass persist=true to enable (the default) or persist=false to disable. Idempotent: if the topic is already configured that way the flag is left as-is and 'changed' is false. This is a WRITE operation.";
}

#[async_trait::async_trait]
impl McpToolCall<SetTopicPersistInput, SetTopicPersistResponse> for SetTopicPersistHandler {
    async fn execute_tool_call(
        &self,
        model: SetTopicPersistInput,
    ) -> Result<SetTopicPersistResponse, String> {
        let persist = model.persist.unwrap_or(true);

        let namespace = self
            .app
            .namespaces
            .get_or_create_optional(model.namespace.as_deref())
            .map_err(|err| format!("Invalid namespace. {}", err))?;

        let topic = namespace
            .topic_list
            .get(&model.topic_id)
            .ok_or_else(|| format!("Topic '{}' not found", model.topic_id))?;

        let previous_persist = topic.get_topic_info(|inner| inner.persist);

        crate::operations::update_topic_persist(&namespace, model.topic_id.clone(), persist)
        .await
        .map_err(|err| {
            format!(
                "Failed to update persist for topic '{}': {:?}",
                model.topic_id, err
            )
        })?;

        Ok(SetTopicPersistResponse {
            topic_id: model.topic_id,
            previous_persist,
            persist,
            changed: previous_persist != persist,
        })
    }
}
