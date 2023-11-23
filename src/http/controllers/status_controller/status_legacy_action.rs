use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use std::sync::Arc;

use crate::app::AppContext;

#[my_http_server::macros::http_route(
    method: "GET",
    route: "/Status",
)]
pub struct GetStatusLegacyAction {
    app: Arc<AppContext>,
}

impl GetStatusLegacyAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

async fn handle_request(
    action: &GetStatusLegacyAction,
    _ctx: &mut HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let result = super::index_models::StatusJsonResult::new(action.app.as_ref()).await;
    return HttpOutput::as_json(result).into_ok_result(true).into();
}
