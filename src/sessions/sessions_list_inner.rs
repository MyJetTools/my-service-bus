use std::{sync::Arc, time::Duration};

use my_tcp_sockets::ConnectionId;

use super::{http::*, tcp::*, MyServiceBusSession, SessionId};

#[cfg(test)]
use super::test::*;

pub struct SessionsListInner {
    snapshot_id: usize,
    tcp_sessions: TcpSessionsList,
    http_sessions: HttpSessionsList,

    #[cfg(test)]
    test_sessions: TestSessionsList,
    current_session_id: i64,
}

impl SessionsListInner {
    pub fn new() -> Self {
        Self {
            snapshot_id: 0,
            current_session_id: 0,
            tcp_sessions: TcpSessionsList::new(),
            #[cfg(test)]
            test_sessions: TestSessionsList::new(),
            http_sessions: HttpSessionsList::new(),
        }
    }

    pub fn get_http_sessions(&self) -> Vec<Arc<MyServiceBusHttpSession>> {
        self.http_sessions.get_all()
    }
    pub fn get_next_session_id(&mut self) -> SessionId {
        let result = self.current_session_id;
        self.current_session_id += 1;
        SessionId::new(result)
    }

    pub fn add_tcp(&mut self, session: Arc<MyServiceBusTcpSession>) {
        self.snapshot_id += 1;

        self.tcp_sessions.add(session)
    }

    pub fn add_http(&mut self, session: Arc<MyServiceBusHttpSession>) {
        self.snapshot_id += 1;

        self.http_sessions.add(session)
    }
    #[cfg(test)]
    pub fn add_test(&mut self, session: Arc<MyServiceBusTestSession>) {
        self.snapshot_id += 1;

        self.test_sessions.add(session)
    }

    pub fn get_tcp_session_by_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<Arc<MyServiceBusTcpSession>> {
        self.tcp_sessions.get_by_connection_id(connection_id)
    }

    pub fn get_session_id_by_tcp_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<SessionId> {
        self.tcp_sessions.get_session_id(connection_id)
    }

    pub fn get_http_by_session_key(
        &self,
        session_key: &str,
    ) -> Option<Arc<MyServiceBusHttpSession>> {
        self.http_sessions.get_by_session_key(session_key)
    }

    pub fn remove_tcp(
        &mut self,
        connection_id: ConnectionId,
    ) -> Option<Arc<MyServiceBusTcpSession>> {
        self.tcp_sessions.remove_by_connection_id(connection_id)
    }

    pub fn remove_http(&mut self, http_session_key: &str) -> Option<Arc<MyServiceBusHttpSession>> {
        self.http_sessions.remove_by_session_key(http_session_key)
    }

    pub fn remove_by_session_id(
        &mut self,
        session_id: SessionId,
    ) -> Option<Arc<dyn MyServiceBusSession + Send + Sync + 'static>> {
        if let Some(result) = self.http_sessions.remove_by_session_id(session_id) {
            return Some(result);
        }

        if let Some(result) = self.tcp_sessions.remove_by_session_id(session_id) {
            return Some(result);
        }

        #[cfg(test)]
        if let Some(result) = self.test_sessions.remove_by_session_id(session_id) {
            return Some(result);
        }

        None
    }

    pub fn get_snapshot(
        &self,
    ) -> (
        usize,
        Vec<Arc<dyn MyServiceBusSession + Send + Sync + 'static>>,
    ) {
        let mut sessions_result =
            Vec::with_capacity(self.tcp_sessions.len() + self.http_sessions.len());

        self.tcp_sessions.fill_sessions(&mut sessions_result);

        self.http_sessions.fill_sessions(&mut sessions_result);

        #[cfg(test)]
        self.test_sessions.fill_sessions(&mut sessions_result);

        return (self.snapshot_id, sessions_result);
    }

    pub fn remove_and_disconnect_expired_http_sessions(
        &mut self,
        inactive_timeout: Duration,
    ) -> Vec<Arc<MyServiceBusHttpSession>> {
        let sessions_to_gc = self.http_sessions.get_sessions_to_gc(inactive_timeout);

        for session_to_gc in &sessions_to_gc {
            self.remove_http(&session_to_gc.session_key.as_str());
        }

        sessions_to_gc
    }
}
