use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult};

use crate::{app::AppContext, sessions::http::MyServiceBusHttpSession};

#[async_trait::async_trait]
pub trait GetSessionToken {
    async fn get_http_session(
        &self,
        ctx: &AppContext,
    ) -> Result<Arc<MyServiceBusHttpSession>, HttpFailResult>;
}
#[async_trait::async_trait]
impl GetSessionToken for HttpContext {
    async fn get_http_session(
        &self,
        ctx: &AppContext,
    ) -> Result<Arc<MyServiceBusHttpSession>, HttpFailResult> {
        if let Some(token) = self.credentials.as_ref() {
            let id = token.get_id();

            if let Some(session) = ctx.sessions.get_http(id).await {
                return Ok(session);
            }
        }

        Err(HttpFailResult::as_unauthorized(None))
    }
}
