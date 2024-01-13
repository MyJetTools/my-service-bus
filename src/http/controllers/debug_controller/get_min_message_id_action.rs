use super::models::*;
use crate::app::AppContext;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use std::sync::Arc;

#[my_http_server::macros::http_route(
    method: "GET",
    route: "/api/Debug/MinMessageId",
    input_data: GetMinMessageIdInputModel,
    description: "Get calculated min message id",
    summary: "Get calculated min message id",
    controller: "Debug",
    result:[
        {status_code: 200, description: "Min messageId", model: MinMessageIdDebugModel},
    ]
)]
pub struct GetMinMessageIdAction {
    app: Arc<AppContext>,
}

impl GetMinMessageIdAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetMinMessageIdAction,
    input_data: GetMinMessageIdInputModel,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let min_message_id = {
        let topic = action.app.topic_list.get(&input_data.topic_id).await;

        if topic.is_none() {
            return Err(HttpFailResult::as_not_found(
                "Topic not found".to_string(),
                false,
            ));
        }

        let topic = topic.unwrap();

        let topic_access = topic.get_access().await;

        topic_access.get_min_message_id()
    };

    let response = MinMessageIdDebugModel {
        min_message_id: if let Some(min_message_id) = min_message_id {
            min_message_id.get_value().into()
        } else {
            None
        },
    };

    HttpOutput::as_json(response).into_ok_result(true).into()
}
