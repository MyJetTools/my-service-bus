use std::sync::Arc;

use my_http_server::macros::http_route;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::{app::AppContext, http::auth::GetSessionToken};

#[http_route(
    method: "POST",
    route: "/api/Greeting/Ping",
    deprecated_routes: ["/Greeting/Ping"],
    controller: "Greeting",
    description: "Ping Http Session",
    summary: "Pings Http Session",

    authorized: "Yes",
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
    ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let connection_data = ctx.get_http_session(&action.app).await?;
    connection_data.ping();
    HttpOutput::Empty.into_ok_result(true).into()
}
