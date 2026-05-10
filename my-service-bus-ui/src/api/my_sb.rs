use crate::models::MySbHttpContract;

const STATUS_PATH: &str = "/api/Status";

pub async fn get_data() -> Result<MySbHttpContract, String> {
    // reqwest's wasm backend rejects relative paths ("builder error" from
    // Url::parse). The SPA is always served from the same origin as the
    // admin API, so anchor against the page's origin.
    let origin = web_sys::window()
        .ok_or_else(|| "no window in current context".to_string())?
        .location()
        .origin()
        .map_err(|e| format!("could not read window.location.origin: {e:?}"))?;
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
