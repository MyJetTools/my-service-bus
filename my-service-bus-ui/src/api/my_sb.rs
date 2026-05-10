use crate::models::MySbHttpContract;

const STATUS_PATH: &str = "/api/Status";
const QUEUES_PATH: &str = "/api/Queues";
const DELETE_TOPIC_PATH: &str = "/api/Topics/Delete";
const RESTORE_TOPIC_PATH: &str = "/api/Topics/Restore";

fn get_origin() -> Result<String, String> {
    web_sys::window()
        .ok_or_else(|| "no window in current context".to_string())?
        .location()
        .origin()
        .map_err(|e| format!("could not read window.location.origin: {e:?}"))
}

pub async fn get_data() -> Result<MySbHttpContract, String> {
    // reqwest's wasm backend rejects relative paths ("builder error" from
    // Url::parse). Anchor against the page's origin — the SPA is always
    // served from the same origin as the admin API.
    let origin = get_origin()?;
    let url = format!("{origin}{STATUS_PATH}");

    let resp = reqwest::get(&url)
        .await
        .map_err(|e| format!("GET {url} failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("GET {url} returned {}", resp.status()));
    }

    resp.json::<MySbHttpContract>()
        .await
        .map_err(|e| format!("decoding {url} response failed: {e}"))
}

pub async fn delete_queue(topic_id: &str, queue_id: &str) -> Result<(), String> {
    let origin = get_origin()?;
    let topic_enc: String = js_sys::encode_uri_component(topic_id).into();
    let queue_enc: String = js_sys::encode_uri_component(queue_id).into();
    let url = format!("{origin}{QUEUES_PATH}?topicId={topic_enc}&queueId={queue_enc}");

    let resp = reqwest::Client::new()
        .delete(&url)
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
    let url = format!(
        "{origin}{DELETE_TOPIC_PATH}?topicId={topic_enc}&hardDeleteMoment={moment_enc}"
    );

    let resp = reqwest::Client::new()
        .delete(&url)
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

    let resp = reqwest::Client::new()
        .put(&url)
        .send()
        .await
        .map_err(|e| format!("PUT {url} failed: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("PUT {url} returned {}", resp.status()));
    }

    Ok(())
}
