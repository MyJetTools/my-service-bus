use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use my_http_server::macros::http_route;

use std::sync::Arc;

use super::*;

use crate::app::AppContext;

#[http_route(
    method: "POST",
    route: "/api/Queues/SetMessageId",
    controller: "Queues",
    description: "Set current queue messageId",
    summary: "Sets current queue messageId",
    input_data: "SetQueueMessageIdInputContract",
    result: [
        {status_code: 202, description: "Operation is succesfull"},
   
    ]
)]
pub struct SetMessageIdAction {
    app: Arc<AppContext>,
}

impl SetMessageIdAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &SetMessageIdAction,
    input_data: SetQueueMessageIdInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {

    crate::operations::queues::set_message_id(
        action.app.as_ref(),
        input_data.topic_id.as_str(),
        input_data.queue_id.as_str(),
        input_data.message_id.into(),
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
