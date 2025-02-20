use std::{sync::Arc, time::Duration};

use my_service_bus::tcp_contracts::MySbTcpConnection;
use my_tcp_sockets::ConnectionId;
use tokio::sync::RwLock;

#[cfg(test)]
use super::test::*;
use super::{
    http::*, sessions_list_inner::SessionsListInner, tcp::*, MyServiceBusSession, SessionId,
};

pub struct SessionsList {
    data: RwLock<SessionsListInner>,
}

impl SessionsList {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(SessionsListInner::new()),
        }
    }

    pub async fn add_tcp(
        &self,
        connection: Arc<MySbTcpConnection>,
        name: String,
        version: Option<String>,
        env_info: Option<String>,
        protocol_version: i32,
    ) {
        let mut write_access = self.data.write().await;

        let session_id = write_access.get_next_session_id();

        let session = MyServiceBusTcpSession::new(
            session_id,
            connection,
            name,
            version,
            env_info,
            protocol_version,
        );
        write_access.add_tcp(Arc::new(session));
    }

    pub async fn add_http(&self, name: String, version: String, ip: String) -> HttpSessionKey {
        let session_key = HttpSessionKey::new();
        let mut write_access = self.data.write().await;
        let session_id = write_access.get_next_session_id();
        let session =
            MyServiceBusHttpSession::new(session_id, session_key.clone(), name, version, ip);
        write_access.add_http(Arc::new(session));
        session_key
    }

    #[cfg(test)]
    pub async fn add_test(
        &self,
        ip: impl Into<rust_extensions::StrOrString<'static>>,
    ) -> Arc<MyServiceBusTestSession> {
        let mut write_access = self.data.write().await;

        let session_id = write_access.get_next_session_id();

        let session = Arc::new(MyServiceBusTestSession::new(session_id, ip));

        write_access.add_test(session.clone());

        session
    }

    pub async fn get_http(&self, http_session_key: &str) -> Option<Arc<MyServiceBusHttpSession>> {
        let read_access = self.data.read().await;
        read_access.get_http_by_session_key(http_session_key)
    }

    pub async fn get_tcp_session_by_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<Arc<MyServiceBusTcpSession>> {
        let read_access = self.data.read().await;
        read_access.get_tcp_session_by_connection_id(connection_id)
    }

    pub async fn get_session_id_by_tcp_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<SessionId> {
        let read_access = self.data.read().await;
        read_access.get_session_id_by_tcp_connection_id(connection_id)
    }

    pub async fn remove_tcp(&self, id: ConnectionId) -> Option<Arc<MyServiceBusTcpSession>> {
        let mut write_access = self.data.write().await;
        write_access.remove_tcp(id)
    }

    pub async fn get_snapshot(
        &self,
    ) -> (
        usize,
        Vec<Arc<dyn MyServiceBusSession + Send + Sync + 'static>>,
    ) {
        let read_access = self.data.read().await;
        read_access.get_snapshot()
    }

    pub async fn one_second_tick(&self) {
        let http_sessions = {
            let read_access = self.data.read().await;
            read_access.get_http_sessions()
        };

        for http_session in http_sessions {
            http_session.one_second_tick().await;
        }
    }

    pub async fn remove_and_disconnect_expired_http_sessions(
        &self,
        inactive_timeout: Duration,
    ) -> Vec<Arc<MyServiceBusHttpSession>> {
        let mut write_access = self.data.write().await;
        write_access.remove_and_disconnect_expired_http_sessions(inactive_timeout)
    }

    pub async fn remove_by_session_id(
        &self,
        session_id: SessionId,
    ) -> Option<Arc<dyn MyServiceBusSession + Send + Sync + 'static>> {
        let mut write_access = self.data.write().await;
        write_access.remove_by_session_id(session_id)
    }
}
