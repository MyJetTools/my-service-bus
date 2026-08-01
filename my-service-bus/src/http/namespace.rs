use std::sync::Arc;

use my_http_server::{HttpContext, HttpFailResult, HttpRequestHeaders};

use crate::{app::AppContext, namespaces::Namespace};

/// Header naming the namespace a request works in. No header — or an empty one —
/// means the default namespace, which is what every pre-namespace client sends.
///
/// Only the admin/read surface uses it. A publisher or a subscriber never does:
/// those carry a session, and a session's namespace is fixed at `/Greeting`, so
/// a forgotten header can not silently send their data somewhere else.
pub const NAMESPACE_HEADER: &str = "ns";

/// Namespace of an incoming admin request. It is created if this is the first
/// time it is mentioned — a namespace is a client-owned name, the same way a
/// topic is.
pub fn get_request_namespace(
    app: &Arc<AppContext>,
    ctx: &HttpContext,
) -> Result<Arc<Namespace>, HttpFailResult> {
    let namespace = match get_namespace_header(ctx) {
        Some(namespace) => Some(namespace),
        None => get_namespace_query_param(ctx),
    };

    match app.namespaces.get_or_create_optional(namespace) {
        Ok(namespace) => Ok(namespace),
        Err(err) => Err(HttpFailResult::as_validation_error(format!(
            "Invalid namespace. {}",
            err
        ))),
    }
}

fn get_namespace_header(ctx: &HttpContext) -> Option<&str> {
    ctx.request
        .get_headers()
        .try_get_case_insensitive_as_str(NAMESPACE_HEADER)
        .ok()
        .flatten()
        .filter(|value| !value.is_empty())
}

/// Fallback for requests which can not carry a header at all — a browser
/// navigation is an `<a href>`, so the UI has no way to attach `ns` to it. The
/// header wins whenever both are present.
fn get_namespace_query_param(ctx: &HttpContext) -> Option<&str> {
    let query = ctx.request.get_uri().query()?;

    for pair in query.split('&') {
        // A valueless element (`?flag`) is somebody else's parameter, not the end
        // of the query string — keep looking.
        let (key, value) = match pair.split_once('=') {
            Some(result) => result,
            None => continue,
        };

        if key.eq_ignore_ascii_case(NAMESPACE_HEADER) {
            if value.is_empty() {
                return None;
            }

            return Some(value);
        }
    }

    None
}
