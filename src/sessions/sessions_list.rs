use std::{sync::Arc, time::Duration};

use my_tcp_sockets::ConnectionId;
use tokio::sync::RwLock;

use super::{
    sessions_list_data::SessionsListData, HttpConnectionData, MyServiceBusSession,
    SessionConnection, TcpConnectionData,
};

pub type SessionId = i64;

pub struct SessionsList {
    data: RwLock<SessionsListData>,
}

impl SessionsList {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(SessionsListData::new()),
        }
    }

    pub async fn add_tcp(&self, data: TcpConnectionData) {
        let mut write_access = self.data.write().await;

        let session = MyServiceBusSession::new(
            write_access.get_next_session_id(),
            SessionConnection::Tcp(data),
        );

        write_access.add(Arc::new(session));
    }

    pub async fn add_http(&self, data: HttpConnectionData) {
        let mut write_access = self.data.write().await;

        let session = MyServiceBusSession::new(
            write_access.get_next_session_id(),
            SessionConnection::Http(data),
        );

        write_access.add(Arc::new(session));
    }

    pub async fn get_http(&self, session_id: &str) -> Option<Arc<MyServiceBusSession>> {
        let read_access = self.data.read().await;
        read_access.get_by_http_session(session_id)
    }

    pub async fn resolve_session_id_by_tcp_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<SessionId> {
        let read_access = self.data.read().await;
        read_access.get_session_id_by_tcp_connection(connection_id)
    }

    pub async fn get(&self, id: SessionId) -> Option<Arc<MyServiceBusSession>> {
        let read_access = self.data.read().await;
        read_access.get(id)
    }

    pub async fn get_by_tcp_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<Arc<MyServiceBusSession>> {
        let read_access = self.data.read().await;
        read_access.get_by_tcp_connection_id(connection_id)
    }

    pub async fn remove_tcp(&self, id: ConnectionId) -> Option<Arc<MyServiceBusSession>> {
        let mut write_access = self.data.write().await;
        write_access.remove_tcp(id)
    }

    pub async fn get_snapshot(&self) -> (usize, Vec<Arc<MyServiceBusSession>>) {
        let read_access = self.data.read().await;
        read_access.get_snapshot()
    }

    pub async fn one_second_tick(&self) {
        let read_access = self.data.read().await;
        read_access.one_second_tick();
    }

    pub async fn remove_and_disconnect_expired_http_sessions(
        &self,
        inactive_timeout: Duration,
    ) -> Option<Vec<Arc<MyServiceBusSession>>> {
        let mut write_access = self.data.write().await;
        let result = write_access.remove_and_disconnect_expired_http_sessions(inactive_timeout);

        if let Some(sessions) = &result {
            for session in sessions {
                session.disconnect().await;
            }
        }

        result
    }
}
