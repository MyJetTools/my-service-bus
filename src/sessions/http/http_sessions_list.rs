use std::{sync::Arc, time::Duration};

use rust_extensions::{
    date_time::DateTimeAsMicroseconds,
    sorted_vec::{EntityWithStrKey, SortedVecOfArcWithStrKey},
};

use super::MyServiceBusHttpSession;

pub struct HttpSessionsList {
    sessions: SortedVecOfArcWithStrKey<MyServiceBusHttpSession>,
}

impl HttpSessionsList {
    pub fn new() -> Self {
        HttpSessionsList {
            sessions: SortedVecOfArcWithStrKey::new(),
        }
    }

    pub fn add(&mut self, session: Arc<MyServiceBusHttpSession>) {
        match self.sessions.insert_or_if_not_exists(session.get_key()) {
            rust_extensions::sorted_vec::InsertIfNotExists::Insert(entry) => {
                entry.insert(session.clone());
            }
            rust_extensions::sorted_vec::InsertIfNotExists::Exists(_) => {
                panic!("Http session with key {} already exists", session.get_key());
            }
        }
    }

    pub fn get(&self, session_key: &str) -> Option<&Arc<MyServiceBusHttpSession>> {
        self.sessions.get(session_key)
    }

    pub fn remove(&mut self, session_key: &str) -> Option<Arc<MyServiceBusHttpSession>> {
        self.sessions.remove(session_key)
    }

    pub fn get_all(&self) -> Vec<Arc<MyServiceBusHttpSession>> {
        self.sessions.iter().map(|itm| itm.clone()).collect()
    }

    pub fn get_sessions_to_gc(
        &self,
        inactive_timeout: Duration,
    ) -> Vec<Arc<MyServiceBusHttpSession>> {
        let mut sessions_to_gc = Vec::new();

        let now = DateTimeAsMicroseconds::now();

        for http_session in self.sessions.iter() {
            let last_incoming = http_session.get_last_incoming_moment();

            if now.duration_since(last_incoming).as_positive_or_zero() > inactive_timeout {
                sessions_to_gc.push(http_session.clone());
            }
        }

        sessions_to_gc
    }
}
