/// Namespace the UI is currently pointed at. Absent means the default namespace,
/// which is what the node assumes when a request carries no `ns` header — so a
/// fresh browser behaves exactly like the UI did before namespaces existed.
const NAMESPACE_KEY: &str = "msb_namespace";

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

/// The api layer reads this on every request, so a page reload restores the
/// selected namespace without anything having to re-apply it.
pub fn load_namespace() -> Option<String> {
    let storage = local_storage()?;
    storage
        .get_item(NAMESPACE_KEY)
        .ok()?
        .filter(|value| !value.is_empty())
}

pub fn save_namespace(namespace: &str) {
    let Some(storage) = local_storage() else {
        return;
    };

    // The default namespace is stored as "nothing selected": the UI then sends no
    // header at all and behaves exactly like a pre-namespace client.
    if namespace.is_empty() {
        let _ = storage.remove_item(NAMESPACE_KEY);
    } else {
        let _ = storage.set_item(NAMESPACE_KEY, namespace);
    }
}
