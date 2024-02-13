use std::sync::Arc;

use crate::sessions::{MyServiceBusSession, SessionId};

use super::MyServiceBusTestSession;

pub struct TestSessionsList {
    items: Vec<Arc<MyServiceBusTestSession>>,
}

impl TestSessionsList {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, session: Arc<MyServiceBusTestSession>) {
        self.items.push(session);
    }

    fn find_by_session_id(&self, session_id: SessionId) -> Option<usize> {
        let mut i = 0;

        for itm in self.items.iter() {
            if itm.session_id == session_id {
                return Some(i);
            }

            i += 1;
        }

        None
    }

    pub fn remove_by_session_id(
        &mut self,
        session_id: SessionId,
    ) -> Option<Arc<MyServiceBusTestSession>> {
        let index = self.find_by_session_id(session_id)?;

        Some(self.items.remove(index))
    }

    pub fn fill_sessions(
        &self,
        dest: &mut Vec<Arc<dyn MyServiceBusSession + Send + Sync + 'static>>,
    ) {
        for itm in self.items.iter() {
            dest.push(itm.clone());
        }
    }
}
