use crate::models::{MySbHttpContract, NamespaceApiModel};

const STATUS_PATH: &str = "/api/Status";
const QUEUES_PATH: &str = "/api/Queues";
const DELETE_TOPIC_PATH: &str = "/api/Topics/Delete";
const RESTORE_TOPIC_PATH: &str = "/api/Topics/Restore";
const NAMESPACES_PATH: &str = "/api/Namespaces/List";
const MCP_WRITES_PATH: &str = "/api/Mcp/Writes";

/// Header naming the namespace a request works in. No header means the default
/// namespace — which is exactly what the UI sends when nothing is selected, so
/// the pre-namespace behaviour is preserved byte for byte.
const NAMESPACE_HEADER: &str = "ns";

fn get_origin() -> Result<String, String> {
    web_sys::window()
        .ok_or_else(|| "no window in current context".to_string())?
        .location()
        .origin()
        .map_err(|e| format!("could not read window.location.origin: {e:?}"))
}

/// Every request is built through here so the namespace can never be forgotten at
/// a call site. The value is read from localStorage on each call rather than
/// threaded through a `Signal`: these are free `async fn`s, not components, so
/// they can not reach into the Dioxus context.
fn request(method: reqwest::Method, url: &str) -> reqwest::RequestBuilder {
    let builder = reqwest::Client::new().request(method, url);

    match crate::storage::load_namespace() {
        Some(namespace) => builder.header(NAMESPACE_HEADER, namespace),
        None => builder,
    }
}

pub async fn get_data() -> Result<MySbHttpContract, String> {
    // reqwest's wasm backend rejects relative paths ("builder error" from
    // Url::parse). Anchor against the page's origin — the SPA is always
    // served from the same origin as the admin API.
    let origin = get_origin()?;
    let url = format!("{origin}{STATUS_PATH}");

    let resp = request(reqwest::Method::GET, &url)
        .send()
        .await
        .map_err(|e| format!("GET {url} failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("GET {url} returned {}", resp.status()));
    }

    resp.json::<MySbHttpContract>()
        .await
        .map_err(|e| format!("decoding {url} response failed: {e}"))
}

/// Deliberately namespace-less: this is the call that tells us which namespaces
/// exist, and the node creates a namespace the moment it sees an unknown name in
/// the header. Sending a stale value here would conjure the very namespace we are
/// trying to check for.
pub async fn get_namespaces_list() -> Result<Vec<NamespaceApiModel>, String> {
    let origin = get_origin()?;
    let url = format!("{origin}{NAMESPACES_PATH}");

    let resp = reqwest::Client::new()
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("GET {url} failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("GET {url} returned {}", resp.status()));
    }

    resp.json::<Vec<NamespaceApiModel>>()
        .await
        .map_err(|e| format!("decoding {url} response failed: {e}"))
}

pub async fn delete_queue(topic_id: &str, queue_id: &str) -> Result<(), String> {
    let origin = get_origin()?;
    let topic_enc: String = js_sys::encode_uri_component(topic_id).into();
    let queue_enc: String = js_sys::encode_uri_component(queue_id).into();
    let url = format!("{origin}{QUEUES_PATH}?topicId={topic_enc}&queueId={queue_enc}");

    let resp = request(reqwest::Method::DELETE, &url)
        .send()
        .await
        .map_err(|e| format!("DELETE {url} failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("DELETE {url} returned {}", resp.status()));
    }

    Ok(())
}

pub async fn delete_topic(topic_id: &str, hard_delete_moment_iso: &str) -> Result<(), String> {
    let origin = get_origin()?;
    let topic_enc: String = js_sys::encode_uri_component(topic_id).into();
    let moment_enc: String = js_sys::encode_uri_component(hard_delete_moment_iso).into();
    let url =
        format!("{origin}{DELETE_TOPIC_PATH}?topicId={topic_enc}&hardDeleteMoment={moment_enc}");

    let resp = request(reqwest::Method::DELETE, &url)
        .send()
        .await
        .map_err(|e| format!("DELETE {url} failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("DELETE {url} returned {}", resp.status()));
    }

    Ok(())
}

pub async fn restore_topic(topic_id: &str) -> Result<(), String> {
    let origin = get_origin()?;
    let topic_enc: String = js_sys::encode_uri_component(topic_id).into();
    let url = format!("{origin}{RESTORE_TOPIC_PATH}?topicId={topic_enc}");

    let resp = request(reqwest::Method::PUT, &url)
        .send()
        .await
        .map_err(|e| format!("PUT {url} failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("PUT {url} returned {}", resp.status()));
    }

    Ok(())
}

/// Opens the MCP-writes window for another 10 minutes, or closes it at once.
/// Namespace-less on purpose: the window is a node-wide switch, not per-namespace.
pub async fn set_mcp_writes(enabled: bool) -> Result<(), String> {
    let origin = get_origin()?;
    let url = format!("{origin}{MCP_WRITES_PATH}?enabled={enabled}");

    let resp = reqwest::Client::new()
        .post(&url)
        .send()
        .await
        .map_err(|e| format!("POST {url} failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("POST {url} returned {}", resp.status()));
    }

    Ok(())
}
