use serde::{Deserialize, Serialize};

/// Name the node gives the namespace every pre-namespace client works in.
pub const DEFAULT_NAMESPACE: &str = "default";

/// Used as the serde default so a session reported by an older node — which sends
/// no `namespace` field at all — still renders as `default` rather than as an
/// empty cell.
pub fn default_namespace() -> String {
    DEFAULT_NAMESPACE.to_string()
}

/// One entry of `GET /api/Namespaces/List`.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct NamespaceApiModel {
    pub name: String,
    #[serde(rename = "topicsAmount")]
    pub topics_amount: usize,
}
