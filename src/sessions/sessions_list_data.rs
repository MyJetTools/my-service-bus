use std::{collections::BTreeMap, sync::Arc, time::Duration};

use my_tcp_sockets::ConnectionId;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::{MyServiceBusSession, SessionConnection, SessionId};

pub struct SessionsListData {
    snapshot_id: usize,
    sessions: BTreeMap<i64, Arc<MyServiceBusSession>>,
    tcp_sessions: BTreeMap<ConnectionId, Arc<MyServiceBusSession>>,
    http_sessions: BTreeMap<String, Arc<MyServiceBusSession>>,
    #[cfg(test)]
    test_sessions: BTreeMap<i64, Arc<MyServiceBusSession>>,
    current_session_id: i64,
}

impl SessionsListData {
    pub fn new() -> Self {
        Self {
            snapshot_id: 0,
            sessions: BTreeMap::new(),
            current_session_id: 0,
            tcp_sessions: BTreeMap::new(),
            #[cfg(test)]
            test_sessions: BTreeMap::new(),
            http_sessions: BTreeMap::new(),
        }
    }
    pub fn get_next_session_id(&mut self) -> SessionId {
        let result = self.current_session_id;
        self.current_session_id += 1;
        SessionId::new(result)
    }

    pub fn add(&mut self, session: Arc<MyServiceBusSession>) {
        self.sessions
            .insert(session.id.get_value(), session.clone());
        self.snapshot_id += 1;

        match &session.connection {
            super::SessionConnection::Tcp(data) => {
                self.tcp_sessions.insert(data.connection.id, session);
            }
            super::SessionConnection::Http(data) => {
                self.http_sessions.insert(data.id.to_string(), session);
            }
            #[cfg(test)]
            super::SessionConnection::Test(connection) => {
                self.test_sessions
                    .insert(connection.id.get_value(), session);
            }
        }
    }

    pub fn get(&self, session_id: SessionId) -> Option<Arc<MyServiceBusSession>> {
        let result = self.sessions.get(&session_id.get_value())?;
        Some(result.clone())
    }

    pub fn get_by_tcp_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<Arc<MyServiceBusSession>> {
        let result = self.tcp_sessions.get(&connection_id)?;
        Some(result.clone())
    }

    pub fn get_by_http_session(&self, session_id: &str) -> Option<Arc<MyServiceBusSession>> {
        let result = self.http_sessions.get(session_id)?;
        Some(result.clone())
    }

    pub fn get_session_id_by_tcp_connection(
        &self,
        connection_id: ConnectionId,
    ) -> Option<SessionId> {
        let result = self.tcp_sessions.get(&connection_id)?;
        Some(result.id)
    }

    fn remove(&mut self, session_id: SessionId) -> Option<Arc<MyServiceBusSession>> {
        let removed_session = self.sessions.remove(&session_id.get_value());

        if let Some(session) = removed_session {
            self.snapshot_id += 1;
            match &session.connection {
                super::SessionConnection::Tcp(data) => {
                    self.tcp_sessions.remove(&data.connection.id);
                }
                super::SessionConnection::Http(data) => {
                    self.http_sessions.remove(&data.id);
                }
                #[cfg(test)]
                super::SessionConnection::Test(connection) => {
                    self.test_sessions.remove(&connection.id.get_value());
                }
            }
            Some(session)
        } else {
            None
        }
    }

    pub fn remove_tcp(&mut self, connection_id: ConnectionId) -> Option<Arc<MyServiceBusSession>> {
        let session_id = self.tcp_sessions.get(&connection_id)?.id;
        return self.remove(session_id);
    }

    pub fn remove_http(&mut self, session_id: &str) -> Option<Arc<MyServiceBusSession>> {
        let session_id = self.http_sessions.get(session_id)?.id;
        return self.remove(session_id);
    }

    pub fn get_snapshot(&self) -> (usize, Vec<Arc<MyServiceBusSession>>) {
        let mut sessions_result = Vec::new();

        for session in self.sessions.values() {
            sessions_result.push(session.clone());
        }

        return (self.snapshot_id, sessions_result);
    }

    pub fn one_second_tick(&self) {
        for session in self.http_sessions.values() {
            if let super::SessionConnection::Http(data) = &session.connection {
                data.one_second_tick();
            }
        }
    }

    pub fn remove_and_disconnect_expired_http_sessions(
        &mut self,
        inactive_timeout: Duration,
    ) -> Option<Vec<Arc<MyServiceBusSession>>> {
        let mut sessions_result = None;

        let now = DateTimeAsMicroseconds::now();

        for session in self.sessions.values() {
            if let SessionConnection::Http(connection) = &session.connection {
                let last_incoming = connection.get_last_incoming_moment();

                if now.duration_since(last_incoming).as_positive_or_zero() > inactive_timeout {
                    if sessions_result.is_none() {
                        sessions_result = Some(Vec::new());
                    }

                    sessions_result.as_mut().unwrap().push(session.clone());
                }
            }
        }

        if let Some(sessions_to_remove) = &sessions_result {
            for session_to_remove in sessions_to_remove {
                if let SessionConnection::Http(connection) = &session_to_remove.connection {
                    self.remove_http(&connection.id);
                }
            }
        }

        return sessions_result;
    }
}
