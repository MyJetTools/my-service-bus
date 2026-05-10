use std::sync::Arc;

use my_tcp_sockets::ConnectionId;
use rust_extensions::sorted_vec::SortedVecOfArc;

use crate::sessions::{MyServiceBusSession, SessionId};

use super::MyServiceBusTcpSession;

pub struct TcpSessionsList {
    by_session_id: SortedVecOfArc<i64, MyServiceBusTcpSession>,
    by_connection_id: SortedVecOfArc<i32, MyServiceBusTcpSession>,
}

impl TcpSessionsList {
    pub fn new() -> Self {
        TcpSessionsList {
            by_session_id: SortedVecOfArc::new(),
            by_connection_id: SortedVecOfArc::new(),
        }
    }

    pub fn add(&mut self, session: Arc<MyServiceBusTcpSession>) {
        match self
            .by_connection_id
            .insert_or_if_not_exists(&session.connection.id)
        {
            rust_extensions::sorted_vec::InsertIfNotExists::Insert(entry) => {
                entry.insert(session.clone())
            }
            rust_extensions::sorted_vec::InsertIfNotExists::Exists(_) => {
                panic!(
                    "Tcp Session already exists with connection id: {}",
                    session.connection.id
                );
            }
        }

        match self
            .by_session_id
            .insert_or_if_not_exists(session.get_session_id().as_ref())
        {
            rust_extensions::sorted_vec::InsertIfNotExists::Insert(entry) => entry.insert(session),
            rust_extensions::sorted_vec::InsertIfNotExists::Exists(_) => {
                panic!(
                    "Tcp Session already exists with session id: {}",
                    session.get_session_id().get_value()
                );
            }
        }
    }

    pub fn get_by_connection_id(
        &self,
        connection_id: ConnectionId,
    ) -> Option<Arc<MyServiceBusTcpSession>> {
        self.by_connection_id.get(&connection_id).cloned()
    }

    pub fn get_session_id(&self, connection_id: ConnectionId) -> Option<SessionId> {
        let result = self.by_connection_id.get(&connection_id)?;
        Some(result.get_session_id())
    }

    pub fn remove_by_session_id(
        &mut self,
        session_id: SessionId,
    ) -> Option<Arc<MyServiceBusTcpSession>> {
        let result = self.by_session_id.remove(session_id.as_ref());

        if let Some(result) = &result {
            self.by_connection_id.remove(&result.connection.id);
        }

        result
    }

    pub fn remove_by_connection_id(
        &mut self,
        connection_id: ConnectionId,
    ) -> Option<Arc<MyServiceBusTcpSession>> {
        let result = self.by_connection_id.remove(&connection_id);

        if let Some(result) = &result {
            self.by_session_id.remove(result.get_session_id().as_ref());
        }

        result
    }

    pub fn fill_sessions(
        &self,
        dest: &mut Vec<Arc<dyn MyServiceBusSession + Send + Sync + 'static>>,
    ) {
        for itm in self.by_session_id.iter() {
            dest.push(itm.clone());
        }
    }

    pub fn len(&self) -> usize {
        self.by_session_id.len()
    }
}
