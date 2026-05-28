use std::sync::Arc;

use my_service_bus::tcp_contracts::MySbTcpConnection;
use tokio::sync::RwLock;

use crate::{client_sessions::ClientSessions, outbound::Outbound};

/// Shared state for the node binary. Built once in `main` and handed to the
/// TCP server callback, the master client callback, and the flusher loop.
pub struct AppContext {
    pub outbound: Arc<Outbound>,
    pub client_sessions: ClientSessions,
    /// The currently-connected master socket. `None` while reconnecting.
    master_connection: RwLock<Option<Arc<MySbTcpConnection>>>,
}

impl AppContext {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            outbound: Outbound::new(),
            client_sessions: ClientSessions::new(),
            master_connection: RwLock::new(None),
        })
    }

    pub async fn set_master_connection(&self, connection: Option<Arc<MySbTcpConnection>>) {
        *self.master_connection.write().await = connection;
    }

    pub async fn get_master_connection(&self) -> Option<Arc<MySbTcpConnection>> {
        self.master_connection.read().await.clone()
    }
}
