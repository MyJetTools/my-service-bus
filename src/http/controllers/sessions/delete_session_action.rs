use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use std::sync::Arc;

use super::*;
use crate::app::AppContext;

#[my_http_server::macros::http_route(
    method: "DELETE",
    route: "/Sessions",
    input_data: "DeleteSessionInputContract",
    summary: "Deletes session",
    description: "Deletes session",
    controller: "Sessions",
    result:[
        {status_code: 202, description: "Successful operation"},
        {status_code: 404, description: "Table not found"},
    ]
)]
pub struct DeleteSessionAction {
    app: Arc<AppContext>,
}

impl DeleteSessionAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &DeleteSessionAction,
    input_data: DeleteSessionInputContract,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    match action
        .app
        .sessions
        .get(input_data.connection_id.into())
        .await
    {
        Some(session) => {
            session.disconnect().await;
            HttpOutput::Empty.into_ok_result(true).into()
        }
        None => Err(HttpFailResult::as_not_found(
            format!("Session {} is not found", input_data.connection_id),
            false,
        )),
    }
}
