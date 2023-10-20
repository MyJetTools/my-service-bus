use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::*;

#[my_http_server::macros::http_route(
    method: "PUT",
    route: "/api/Topics/Restore",
    description: "Restore topic which had not been hard deleted yet",
    summary: "Restores topic which had not been hard deleted yet",
    input_data: "RestoreTopicRequestContract",
    controller: "Topics",
    result:[
        {status_code: 202, description: "Topic is soft deleted", model:"RestoreTopicResponseContract"},
    ]
)]
pub struct RestoreTopicAction {
    app: Arc<AppContext>,
}

impl RestoreTopicAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &RestoreTopicAction,
    input_data: RestoreTopicRequestContract,
    _: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let restored = crate::operations::restore_topic(&action.app, &input_data.topic_id).await;
    let response = RestoreTopicResponseContract { restored };
    HttpOutput::as_json(response).into_ok_result(true).into()
}
