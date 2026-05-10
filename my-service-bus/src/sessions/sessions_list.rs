use std::{
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc, Mutex,
    },
    time::Duration,
};

use arc_swap::ArcSwap;
use my_service_bus::tcp_contracts::MySbTcpConnection;
use my_tcp_sockets::ConnectionId;
use rust_extensions::sorted_vec::SortedVec;

#[cfg(test)]
use super::test::*;
use super::{http::*, tcp::*, MyServiceBusSession, MyServiceBusSessionInner, SessionId};

#[derive(Clone)]
struct SessionsInner {
    snapshot_id: usize,
    by_session_id: SortedVec<i64, MyServiceBusSession>,
    tcp: Vec<Arc<MyServiceBusTcpSession>>,
    http: HttpSessionsList,
}

impl SessionsInner {
    fn new() -> Self {
        Self {
            snapshot_id: 0,
            by_session_id: SortedVec::new(),
            tcp: Vec::new(),
            http: HttpSessionsList::new(),
        }
    }
}

fn find_tcp(
    tcp: &[Arc<MyServiceBusTcpSession>],
    connection_id: ConnectionId,
) -> Result<usize, usize> {
    tcp.binary_search_by(|s| s.connection.id.cmp(&connection_id))
}

pub struct SessionsList {
    inner: ArcSwap<SessionsInner>,
    write_lock: Mutex<()>,
    next_session_id: AtomicI64,
}

impl SessionsList {
    pub fn new() -> Self {
        Self {
            inner: ArcSwap::from_pointee(SessionsInner::new()),
            write_lock: Mutex::new(()),
            next_session_id: AtomicI64::new(0),
        }
    }

    fn get_next_session_id(&self) -> SessionId {
        SessionId::new(self.next_session_id.fetch_add(1, Ordering::Relaxed))
    }

    pub fn add_tcp(
        &self,
        connection: Arc<MySbTcpConnection>,
        name: String,
        version: Option<String>,
        env_info: Option<String>,
        protocol_version: i32,
    ) {
        let session_id = self.get_next_session_id();

        let tcp_session = Arc::new(MyServiceBusTcpSession::new(
            session_id,
            connection,
            name,
            version,
            env_info,
            protocol_version,
        ));

        let session = MyServiceBusSession {
            session_id,
            inner: MyServiceBusSessionInner::Tcp(tcp_session.clone()),
        };

        let _guard = self.write_lock.lock().unwrap();
        let current = self.inner.load_full();
        let mut new_inner = (*current).clone();

        let connection_id = tcp_session.connection.id;
        match find_tcp(&new_inner.tcp, connection_id) {
            Ok(idx) => new_inner.tcp[idx] = tcp_session,
            Err(idx) => new_inner.tcp.insert(idx, tcp_session),
        }

        new_inner.by_session_id.insert_or_replace(session);
        new_inner.snapshot_id += 1;

        self.inner.store(Arc::new(new_inner));
    }

    pub fn add_http(&self, name: String, version: String, ip: String) -> HttpSessionKey {
        let session_key = HttpSessionKey::new();
        let session_id = self.get_next_session_id();
        let http_session = Arc::new(MyServiceBusHttpSession::new(
            session_id,
            session_key.clone(),
            name,
            version,
            ip,
        ));
        let session = MyServiceBusSession {
            session_id,
            inner: MyServiceBusSessionInner::Http(http_session.clone()),
        };

        let _guard = self.write_lock.lock().unwrap();
        let current = self.inner.load_full();
        let mut new_inner = (*current).clone();

        new_inner.http.add(http_session);
        new_inner.by_session_id.insert_or_replace(session);
        new_inner.snapshot_id += 1;

        self.inner.store(Arc::new(new_inner));

        session_key
    }

    #[cfg(test)]
    pub fn add_test(&self) -> Arc<MyServiceBusTestSession> {
        let session_id = self.get_next_session_id();
        let test_session = Arc::new(MyServiceBusTestSession::new(session_id));
        let session = MyServiceBusSession {
            session_id,
            inner: MyServiceBusSessionInner::Test(test_session.clone()),
        };

        let _guard = self.write_lock.lock().unwrap();
        let current = self.inner.load_full();
        let mut new_inner = (*current).clone();

        new_inner.by_session_id.insert_or_replace(session);
        new_inner.snapshot_id += 1;

        self.inner.store(Arc::new(new_inner));

        test_session
    }

    pub fn get_http(&self, http_session_key: &str) -> Option<Arc<MyServiceBusHttpSession>> {
        self.inner.load().http.get(http_session_key).cloned()
    }

    pub fn get_tcp_session_by_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<Arc<MyServiceBusTcpSession>> {
        let inner = self.inner.load();
        match find_tcp(&inner.tcp, connection_id) {
            Ok(idx) => Some(inner.tcp[idx].clone()),
            Err(_) => None,
        }
    }

    pub fn get_session_id_by_tcp_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<SessionId> {
        let inner = self.inner.load();
        match find_tcp(&inner.tcp, connection_id) {
            Ok(idx) => Some(inner.tcp[idx].session_id),
            Err(_) => None,
        }
    }

    pub fn remove_tcp(&self, connection_id: ConnectionId) -> Option<MyServiceBusSession> {
        let _guard = self.write_lock.lock().unwrap();
        let current = self.inner.load_full();
        let mut new_inner = (*current).clone();

        let tcp_idx = find_tcp(&new_inner.tcp, connection_id).ok()?;
        let removed_tcp = new_inner.tcp.remove(tcp_idx);
        let removed_session = new_inner
            .by_session_id
            .remove(removed_tcp.session_id.as_ref())?;
        new_inner.snapshot_id += 1;

        self.inner.store(Arc::new(new_inner));

        Some(removed_session)
    }

    pub fn get_snapshot(&self) -> (usize, Vec<MyServiceBusSession>) {
        let inner = self.inner.load();
        (inner.snapshot_id, inner.by_session_id.as_slice().to_vec())
    }

    pub async fn one_second_tick(&self) {
        let http_sessions = self.inner.load().http.get_all();

        for http_session in http_sessions {
            http_session.one_second_tick().await;
        }
    }

    pub fn remove_and_disconnect_expired_http_sessions(
        &self,
        inactive_timeout: Duration,
    ) -> Vec<Arc<MyServiceBusHttpSession>> {
        let _guard = self.write_lock.lock().unwrap();
        let current = self.inner.load_full();
        let sessions_to_gc = current.http.get_sessions_to_gc(inactive_timeout);

        if sessions_to_gc.is_empty() {
            return sessions_to_gc;
        }

        let mut new_inner = (*current).clone();
        for session_to_gc in &sessions_to_gc {
            if let Some(removed) = new_inner.http.remove(session_to_gc.session_key.as_str()) {
                new_inner.by_session_id.remove(removed.session_id.as_ref());
            }
        }
        new_inner.snapshot_id += 1;

        self.inner.store(Arc::new(new_inner));

        sessions_to_gc
    }

    pub fn remove_by_session_id(&self, session_id: SessionId) -> Option<MyServiceBusSession> {
        let _guard = self.write_lock.lock().unwrap();
        let current = self.inner.load_full();
        let mut new_inner = (*current).clone();

        let removed = new_inner.by_session_id.remove(session_id.as_ref())?;

        match &removed.inner {
            MyServiceBusSessionInner::Tcp(session) => {
                if let Ok(idx) = find_tcp(&new_inner.tcp, session.connection.id) {
                    new_inner.tcp.remove(idx);
                }
            }
            MyServiceBusSessionInner::Http(session) => {
                new_inner.http.remove(session.session_key.as_str());
            }
            #[cfg(test)]
            MyServiceBusSessionInner::Test(_) => {}
        }
        new_inner.snapshot_id += 1;

        self.inner.store(Arc::new(new_inner));

        Some(removed)
    }
}
