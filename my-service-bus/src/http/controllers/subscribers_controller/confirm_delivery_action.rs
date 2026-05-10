use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, http::auth::GetSessionToken};

use super::contracts::*;

#[my_http_server::macros::http_route(
    method: "POST",
    route: "/api/Subscribers/Confirm",
    input_data: "ConfirmDeliveryHttpModel",
    description: "Subscribe to queue",
    summary: "Subscribe to queue",
    controller: "Subscribers",
    result:[
        {status_code: 202, description: "Confirmed"},
    ]
)]
pub struct ConfirmDeliveryAction {
    app: Arc<AppContext>,
}

impl ConfirmDeliveryAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &ConfirmDeliveryAction,
    input_data: ConfirmDeliveryHttpModel,
    ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let http_session = ctx.get_http_session(&action.app).await?;

    http_session.ping();

    if let Some(confirmations) = input_data.confirmation {
        for confirmation in confirmations {
            if confirmation.all_confirmed_ok() {
                crate::operations::delivery_confirmation::all_confirmed(
                    &action.app,
                    &confirmation.topic_id,
                    &confirmation.queue_id,
                    confirmation.subscriber_id.into(),
                )
                .await?;
            }
        }
    }

    HttpOutput::Empty.into_ok_result(true).into()
}
