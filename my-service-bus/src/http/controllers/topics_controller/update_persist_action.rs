use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::*;

#[my_http_server::macros::http_route(
    method: "POST",
    route: "/api/Topics/Persist",
    input_data: UpdatePersistRequestContract,
    description: "Update persist topic property",
    summary: "Update persist topic property",
    controller: "Topics",
    result:[
        {status_code: 202, description: "Topic persist flag is updated"},
    ]
)]
pub struct UpdatePersistAction {
    app: Arc<AppContext>,
}

impl UpdatePersistAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &UpdatePersistAction,
    input_data: UpdatePersistRequestContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    crate::operations::update_topic_persist(&action.app, input_data.topic_id, input_data.persist)
        .await?;

    HttpOutput::as_text("Topic is created".to_string())
        .into_ok_result(true)
        .into()
}
