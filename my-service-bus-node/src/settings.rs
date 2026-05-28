use serde::{Deserialize, Serialize};

/// Loaded from `~/.myservicebusnode` (YAML).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsModel {
    /// `host:port` of the master my-service-bus TCP server.
    #[serde(rename = "MasterTcpUrl")]
    pub master_tcp_url: String,
    /// `host:port` for the node's local TCP server (where downstream clients
    /// connect).
    #[serde(rename = "ListenTcp")]
    pub listen_tcp: String,
}

impl SettingsModel {
    pub fn load() -> Self {
        let home = std::env::var("HOME").expect("HOME env var is required");
        let path = format!("{}/.myservicebusnode", home);
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Cannot read settings file {}: {}", path, e));
        serde_yaml::from_str(&raw)
            .unwrap_or_else(|e| panic!("Cannot parse settings file {}: {}", path, e))
    }
}
