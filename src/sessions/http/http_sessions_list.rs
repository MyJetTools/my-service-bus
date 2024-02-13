use std::{sync::Arc, time::Duration};

use rust_extensions::{
    date_time::DateTimeAsMicroseconds,
    sorted_vec::{EntityWithStrKey, SortedVecOfArc, SortedVecOfArcWithStrKey},
};

use crate::sessions::{MyServiceBusSession, SessionId};

use super::MyServiceBusHttpSession;

pub struct HttpSessionsList {
    by_session_key: SortedVecOfArcWithStrKey<MyServiceBusHttpSession>,
    by_session_id: SortedVecOfArc<i64, MyServiceBusHttpSession>,
}

impl HttpSessionsList {
    pub fn new() -> Self {
        HttpSessionsList {
            by_session_key: SortedVecOfArcWithStrKey::new(),
            by_session_id: SortedVecOfArc::new(),
        }
    }

    pub fn add(&mut self, session: Arc<MyServiceBusHttpSession>) {
        match self
            .by_session_key
            .insert_or_if_not_exists(session.get_key())
        {
            rust_extensions::sorted_vec::InsertIfNotExists::Insert(entry) => {
                entry.insert(session.clone());
            }
            rust_extensions::sorted_vec::InsertIfNotExists::Exists(_) => {
                panic!("Http session with key {} already exists", session.get_key());
            }
        }

        match self
            .by_session_id
            .insert_or_if_not_exists(session.session_id.as_ref())
        {
            rust_extensions::sorted_vec::InsertIfNotExists::Insert(entry) => {
                entry.insert(session);
            }
            rust_extensions::sorted_vec::InsertIfNotExists::Exists(_) => {
                panic!("Http session with key {} already exists", session.get_key());
            }
        }
    }

    pub fn get_by_session_key(&self, session_key: &str) -> Option<Arc<MyServiceBusHttpSession>> {
        self.by_session_key.get(session_key).cloned()
    }

    pub fn remove_by_session_id(
        &mut self,
        session_id: SessionId,
    ) -> Option<Arc<MyServiceBusHttpSession>> {
        let result = self.by_session_id.remove(session_id.as_ref());

        if let Some(result) = &result {
            self.by_session_key.remove(result.session_key.as_str());
        }

        result
    }

    pub fn remove_by_session_key(
        &mut self,
        session_key: &str,
    ) -> Option<Arc<MyServiceBusHttpSession>> {
        let result = self.by_session_key.remove(session_key);

        if let Some(result) = &result {
            self.by_session_id.remove(result.session_id.as_ref());
        }

        result
    }

    pub fn get_all(&self) -> Vec<Arc<MyServiceBusHttpSession>> {
        self.by_session_id.iter().map(|itm| itm.clone()).collect()
    }

    pub fn fill_sessions(
        &self,
        dest: &mut Vec<Arc<dyn MyServiceBusSession + Send + Sync + 'static>>,
    ) {
        for itm in self.by_session_id.iter() {
            dest.push(itm.clone());
        }
    }

    pub fn get_sessions_to_gc(
        &self,
        inactive_timeout: Duration,
    ) -> Vec<Arc<MyServiceBusHttpSession>> {
        let mut sessions_to_gc = Vec::new();

        let now = DateTimeAsMicroseconds::now();

        for http_session in self.by_session_id.iter() {
            let last_incoming = http_session.get_last_incoming_moment();

            if now.duration_since(last_incoming).as_positive_or_zero() > inactive_timeout {
                sessions_to_gc.push(http_session.clone());
            }
        }

        sessions_to_gc
    }

    pub fn len(&self) -> usize {
        self.by_session_id.len()
    }
}
