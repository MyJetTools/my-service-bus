use my_http_server::*;

const AUTH_HEADER: &str = "authorization";

pub struct SessionToken {
    pub session: String,
}

impl RequestCredentials for SessionToken {
    fn get_id(&self) -> &str {
        &self.session
    }

    fn get_claims<'s>(&'s self) -> Option<Vec<RequestClaim<'s>>> {
        None
    }
}

pub struct AuthMiddleware;

#[async_trait::async_trait]
impl HttpServerMiddleware for AuthMiddleware {
    async fn handle_request(
        &self,
        ctx: &mut HttpContext,
        get_next: &mut HttpServerRequestFlow,
    ) -> Result<HttpOkResult, HttpFailResult> {
        if let Some(header) = ctx.request.get_header(AUTH_HEADER) {
            let token = SessionToken {
                session: header.to_string(),
            };
            ctx.credentials = Some(Box::new(token));
        }

        get_next.next(ctx).await
    }
}
