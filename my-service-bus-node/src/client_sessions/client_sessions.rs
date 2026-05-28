use std::sync::Arc;

use my_service_bus::tcp_contracts::MySbTcpConnection;
use tokio::sync::RwLock;

use super::client_sessions_inner::ClientSessionsInner;

/// Registry of locally connected clients (each has its own TCP connection to
/// the node's listen port). The node uses the TCP `connection_id` (set by
/// my-tcp-sockets) as the stable handle to a client during its lifetime.
pub struct ClientSessions {
    inner: RwLock<ClientSessionsInner>,
}

impl ClientSessions {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(ClientSessionsInner::new()),
        }
    }

    pub async fn add(&self, connection: Arc<MySbTcpConnection>) {
        self.inner.write().await.add(connection);
    }

    pub async fn remove(&self, connection_id: i32) -> Option<Arc<MySbTcpConnection>> {
        self.inner.write().await.remove(connection_id)
    }

    pub async fn get(&self, connection_id: i32) -> Option<Arc<MySbTcpConnection>> {
        self.inner.read().await.get(connection_id)
    }
}

impl Default for ClientSessions {
    fn default() -> Self {
        Self::new()
    }
}
