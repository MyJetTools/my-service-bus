use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, http::auth::GetSessionToken};

use super::contracts::*;

#[my_http_server::macros::http_route(
    method: "POST",
    route: "/api/Subscribers/Subscribe",
    input_data: "SubscribeHttpInputModel",
    description: "Subscribe to queue",
    summary: "Subscribe to queue",
    controller: "Subscribers",
    result:[
        {status_code: 202, description: "Subscribed"},
    ]
)]
pub struct SubscribeAction {
    app: Arc<AppContext>,
}

impl SubscribeAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &SubscribeAction,
    input_data: SubscribeHttpInputModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let session = ctx.get_http_session(&action.app).await?;

    session.connection.unwrap_as_http().ping();

    let queue_type = input_data.get_queue_type();
    crate::operations::subscriber::subscribe_to_queue(
        &action.app,
        input_data.topic_id,
        input_data.queue_id,
        queue_type,
        &session,
    )
    .await?;

    HttpOutput::Empty.into_ok_result(true).into()
}
