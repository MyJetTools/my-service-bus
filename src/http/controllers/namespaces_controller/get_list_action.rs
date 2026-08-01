use std::sync::Arc;

use my_http_server::macros::http_route;
use my_http_server::{HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use crate::app::AppContext;

use super::models::NamespaceContract;

#[http_route(
    method: "GET",
    route: "/api/Namespaces/List",
    controller: "Namespaces",
    description: "Get list of namespaces",
    summary: "Returns every namespace of this node together with the amount of topics in it",
    result: [
        {status_code: 200, description: "List of namespaces", model: "Vec<NamespaceContract>"},
    ]
)]
pub struct GetNamespacesListAction {
    app: Arc<AppContext>,
}

impl GetNamespacesListAction {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

/// Deliberately namespace-less: this is the call that tells the UI which
/// namespaces exist, and the node creates one the moment it sees an unknown name
/// in the `ns` header. Honouring a stale header here would conjure the very
/// namespace the caller is checking for.
async fn handle_request(
    action: &GetNamespacesListAction,
    _ctx: &HttpContext,
) -> Result<HttpOkResult, HttpFailResult> {
    let mut result = Vec::new();

    for namespace in action.app.namespaces.get_all().iter() {
        result.push(NamespaceContract {
            name: namespace.name.to_string(),
            topics_amount: namespace.topic_list.get_all().len(),
        });
    }

    HttpOutput::as_json(result).into_ok_result(true).into()
}
