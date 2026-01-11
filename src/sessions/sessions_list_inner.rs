use std::{sync::Arc, time::Duration};

use my_tcp_sockets::ConnectionId;
use rust_extensions::sorted_vec::*;

use super::{http::*, tcp::*, MyServiceBusSession, SessionId};

pub struct SessionsListInner {
    snapshot_id: usize,
    by_session_id: SortedVec<i64, MyServiceBusSession>,
    tcp: SortedVecOfArc<i32, MyServiceBusTcpSession>,
    http: HttpSessionsList,

    current_session_id: i64,
}

impl SessionsListInner {
    pub fn new() -> Self {
        Self {
            snapshot_id: 0,
            current_session_id: 0,
            by_session_id: SortedVec::new(),
            tcp: SortedVecOfArc::new(),
            http: HttpSessionsList::new(),
        }
    }

    pub fn get_http_sessions(&self) -> Vec<Arc<MyServiceBusHttpSession>> {
        self.http.get_all()
    }

    pub fn get_next_session_id(&mut self) -> SessionId {
        let result = self.current_session_id;
        self.current_session_id += 1;
        SessionId::new(result)
    }

    pub fn add(&mut self, session: MyServiceBusSession) {
        self.snapshot_id += 1;

        match &session.inner {
            super::MyServiceBusSessionInner::Tcp(session) => {
                self.tcp.insert_or_replace(session.clone());
            }
            super::MyServiceBusSessionInner::Http(session) => {
                self.http.add(session.clone());
            }
            #[cfg(test)]
            super::MyServiceBusSessionInner::Test(_) => {}
        }

        self.by_session_id.insert_or_replace(session);
    }

    pub fn get_tcp_session_by_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<Arc<MyServiceBusTcpSession>> {
        self.tcp.get(&connection_id).cloned()
    }

    pub fn get_session_id_by_tcp_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<SessionId> {
        let session = self.tcp.get(&connection_id)?;
        Some(session.session_id)
    }

    pub fn get_http_by_session_key(
        &self,
        session_key: &str,
    ) -> Option<Arc<MyServiceBusHttpSession>> {
        self.http.get(session_key).cloned()
    }

    pub fn remove_tcp(&mut self, connection_id: ConnectionId) -> Option<MyServiceBusSession> {
        let removed = self.tcp.remove(&connection_id);

        if let Some(removed) = removed.as_ref() {
            return self.by_session_id.remove(removed.session_id.as_ref());
        }

        None
    }

    pub fn remove_http(&mut self, http_session_key: &str) -> Option<Arc<MyServiceBusHttpSession>> {
        let removed_session = self.http.remove(http_session_key);

        if let Some(removed) = removed_session.as_ref() {
            self.by_session_id.remove(removed.session_id.as_ref());
        }

        removed_session
    }

    pub fn remove_by_session_id(&mut self, session_id: SessionId) -> Option<MyServiceBusSession> {
        let removed = self.by_session_id.remove(session_id.as_ref());

        if let Some(removed) = &removed {
            match &removed.inner {
                super::MyServiceBusSessionInner::Tcp(session) => {
                    self.tcp.remove(&session.connection.id);
                }
                super::MyServiceBusSessionInner::Http(session) => {
                    self.http.remove(session.session_key.as_str());
                }
                #[cfg(test)]
                super::MyServiceBusSessionInner::Test(_) => {}
            }
        }

        removed
    }

    pub fn get_snapshot(&self) -> (usize, Vec<MyServiceBusSession>) {
        return (self.snapshot_id, self.by_session_id.as_slice().to_vec());
    }

    pub fn remove_and_disconnect_expired_http_sessions(
        &mut self,
        inactive_timeout: Duration,
    ) -> Vec<Arc<MyServiceBusHttpSession>> {
        let sessions_to_gc = self.http.get_sessions_to_gc(inactive_timeout);

        for session_to_gc in &sessions_to_gc {
            self.remove_http(&session_to_gc.session_key.as_str());
        }

        sessions_to_gc
    }
}
