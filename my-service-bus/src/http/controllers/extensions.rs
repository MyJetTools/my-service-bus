use async_trait::async_trait;
use std::sync::Arc;

use my_http_server::HttpFailResult;

use crate::{app::AppContext, sessions::MyServiceBusSession};

#[async_trait]
pub trait HttpContextExtensions {
    async fn get_http_session(
        &self,
        session_id: &str,
    ) -> Result<Arc<MyServiceBusSession>, HttpFailResult>;
}

#[async_trait]
impl HttpContextExtensions for AppContext {
    async fn get_http_session(
        &self,
        session_id: &str,
    ) -> Result<Arc<MyServiceBusSession>, HttpFailResult> {
        match self.sessions.get_http(session_id) {
            Some(session) => {
                if session.connection.is_http() {
                    Ok(session)
                } else {
                    Err(HttpFailResult::as_unauthorized(Some(
                        "Session should has HTTP Type".to_string(),
                    )))
                }
            }
            None => Err(HttpFailResult::as_unauthorized(None)),
        }
    }
}
