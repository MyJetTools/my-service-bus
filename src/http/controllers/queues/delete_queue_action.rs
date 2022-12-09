use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use my_http_server_swagger::http_route;

use std::sync::Arc;

use super::*;

use crate::app::AppContext;

#[http_route(
    method: "DELETE",
    route: "/Queues",
    controller: "Queues",
    summary: "Delete a queue",
    description: "Delets queue",
    input_data: "DeleteQueueInputContract",
    result: [
        {status_code: 204, description: "Queue is deleted"},
             {status_code: 404, description: "Topic or Queue is not found"}
    ]
)]
pub struct DeleteQueueAction {
    app: Arc<AppContext>,
}

impl DeleteQueueAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &DeleteQueueAction,
    http_input: DeleteQueueInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::operations::queues::delete_queue(
        action.app.as_ref(),
        http_input.topic_id.as_str(),
        http_input.queue_id.as_str(),
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
