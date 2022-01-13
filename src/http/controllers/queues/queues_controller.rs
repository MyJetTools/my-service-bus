use async_trait::async_trait;
use my_service_bus_shared::MessageId;
use std::sync::Arc;

use my_http_server::{
    middlewares::controllers::{
        actions::{DeleteAction, GetAction, PostAction},
        documentation::{
            HttpActionDescription, HttpInputParameter, HttpParameterInputSource, HttpParameterType,
        },
    },
    HttpContext, HttpFailResult, HttpOkResult, WebContentType,
};

use crate::app::AppContext;
pub struct QueuesController {
    app: Arc<AppContext>,
}

impl QueuesController {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait]
impl GetAction for QueuesController {
    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Queues",
            description: "Set list of queues",
            out_content_type: WebContentType::Json,
            input_params: Some(vec![HttpInputParameter {
                name: "topicId".to_string(),
                param_type: HttpParameterType::String,
                description: "Id of topic".to_string(),
                source: HttpParameterInputSource::Query,
                required: true,
            }]),
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;
        let topic_id = query.get_required_string_parameter("topicId")?;

        let topic = self.app.topic_list.get(topic_id).await;

        if topic.is_none() {
            return Err(HttpFailResult::as_not_found(
                format!("Topic {} not found", topic_id),
                false,
            ));
        }

        let topic = topic.unwrap();

        let mut result = Vec::new();

        {
            let topic_data = topic.get_access("http.get_queues").await;
            for queue in topic_data.queues.get_queues() {
                result.push(queue.queue_id.clone());
            }
        }

        return HttpOkResult::create_json_response(result).into();
    }
}

#[async_trait]
impl DeleteAction for QueuesController {
    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Queues",
            description: "Delete queue",
            out_content_type: WebContentType::Json,
            input_params: Some(vec![
                HttpInputParameter {
                    name: "topicId".to_string(),
                    param_type: HttpParameterType::String,
                    description: "Id of topic".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
                HttpInputParameter {
                    name: "queueId".to_string(),
                    param_type: HttpParameterType::String,
                    description: "Id of queue".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
            ]),
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;

        let topic_id = query.get_required_string_parameter("topicId")?;
        let queue_id = query.get_required_string_parameter("queueId")?;

        crate::operations::queues::delete_queue(self.app.as_ref(), topic_id, queue_id).await?;

        Ok(HttpOkResult::Ok)
    }
}

#[async_trait]
impl PostAction for QueuesController {
    fn get_description(&self) -> Option<HttpActionDescription> {
        HttpActionDescription {
            name: "Queues",
            description: "Set message id of the queue",
            out_content_type: WebContentType::Json,
            input_params: Some(vec![
                HttpInputParameter {
                    name: "topicId".to_string(),
                    param_type: HttpParameterType::String,
                    description: "Id of topic".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
                HttpInputParameter {
                    name: "queueId".to_string(),
                    param_type: HttpParameterType::String,
                    description: "Id of queue".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
                HttpInputParameter {
                    name: "messageId".to_string(),
                    param_type: HttpParameterType::Long,
                    description: "Id of message".to_string(),
                    source: HttpParameterInputSource::Query,
                    required: true,
                },
            ]),
        }
        .into()
    }

    async fn handle_request(&self, ctx: HttpContext) -> Result<HttpOkResult, HttpFailResult> {
        let query = ctx.get_query_string()?;
        let topic_id = query.get_required_string_parameter("topicId")?;
        let queue_id = query.get_required_string_parameter("queueId")?;
        let message_id: MessageId = query.get_required_parameter("messageId")?;

        crate::operations::queues::set_message_id(
            self.app.as_ref(),
            topic_id,
            queue_id,
            message_id,
        )
        .await?;

        Ok(HttpOkResult::Ok)
    }
}