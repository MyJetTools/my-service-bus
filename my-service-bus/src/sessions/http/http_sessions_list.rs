use std::{sync::Arc, time::Duration};

use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::MyServiceBusHttpSession;

#[derive(Clone)]
pub struct HttpSessionsList {
    sessions: Vec<Arc<MyServiceBusHttpSession>>,
}

fn find(sessions: &[Arc<MyServiceBusHttpSession>], session_key: &str) -> Result<usize, usize> {
    sessions.binary_search_by(|s| s.session_key.as_str().cmp(session_key))
}

impl HttpSessionsList {
    pub fn new() -> Self {
        HttpSessionsList {
            sessions: Vec::new(),
        }
    }

    pub fn add(&mut self, session: Arc<MyServiceBusHttpSession>) {
        match find(&self.sessions, session.session_key.as_str()) {
            Ok(_) => {
                panic!(
                    "Http session with key {} already exists",
                    session.session_key.as_str()
                );
            }
            Err(idx) => {
                self.sessions.insert(idx, session);
            }
        }
    }

    pub fn get(&self, session_key: &str) -> Option<&Arc<MyServiceBusHttpSession>> {
        match find(&self.sessions, session_key) {
            Ok(idx) => Some(&self.sessions[idx]),
            Err(_) => None,
        }
    }

    pub fn remove(&mut self, session_key: &str) -> Option<Arc<MyServiceBusHttpSession>> {
        let idx = find(&self.sessions, session_key).ok()?;
        Some(self.sessions.remove(idx))
    }

    pub fn get_all(&self) -> Vec<Arc<MyServiceBusHttpSession>> {
        self.sessions.clone()
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
