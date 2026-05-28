use std::sync::Arc;

use ahash::AHashMap;
use my_service_bus::tcp_contracts::MySbTcpConnection;

pub(super) struct ClientSessionsInner {
    by_connection_id: AHashMap<i32, Arc<MySbTcpConnection>>,
}

impl ClientSessionsInner {
    pub(super) fn new() -> Self {
        Self {
            by_connection_id: AHashMap::new(),
        }
    }

    pub(super) fn add(&mut self, connection: Arc<MySbTcpConnection>) {
        self.by_connection_id.insert(connection.id, connection);
    }

    pub(super) fn remove(&mut self, connection_id: i32) -> Option<Arc<MySbTcpConnection>> {
        self.by_connection_id.remove(&connection_id)
    }

    pub(super) fn get(&self, connection_id: i32) -> Option<Arc<MySbTcpConnection>> {
        self.by_connection_id.get(&connection_id).cloned()
    }
}
