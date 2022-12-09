use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};
use my_http_server_swagger::http_route;

use crate::{app::AppContext, http::controllers::extensions::HttpContextExtensions};

use super::models::PingInputModel;

#[http_route(
    method: "POST",
    route: "/Greeting/Ping",
    controller: "Greeting",
    summary: "Ping Http Session",
    description: "Pings Http Session",
    input_data: "PingInputModel",
    ok_result_description: "Session is alive",
    result: [
        {status_code: 202, description: "Session description"},
        {status_code: 400, description: "Bad request"}, 
        {status_code: 401, description: "Unauthorized"},
    ]
)]
pub struct PingAction {
    app: Arc<AppContext>,
}

impl PingAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &PingAction,
    input_data: PingInputModel,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let http_session = action
        .app
        .get_http_session(input_data.authorization.as_str())
        .await?;

    http_session.as_ref().connection.unwrap_as_http().ping();

    HttpOutput::Empty.into_ok_result(true).into()
}
