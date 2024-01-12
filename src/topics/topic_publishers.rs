use crate::sessions::SessionId;

pub struct TopicPublishers {
    inner: Vec<(i64, u8)>,
}

impl TopicPublishers {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    pub fn add(&mut self, session_id: SessionId, value: u8) {
        if self.has_session(session_id) {
            return;
        }
        self.inner.push((session_id.get_value(), value));
    }

    pub fn has_session(&self, session_id: SessionId) -> bool {
        self.inner.iter().any(|x| x.0 == session_id.get_value())
    }

    pub fn one_second_tick(&mut self) {
        for value in &mut self.inner {
            if value.1 > 0 {
                value.1 -= 1;
            }
        }
    }

    pub fn remove(&mut self, session_id: SessionId) {
        self.inner.retain(|x| x.0 != session_id.get_value());
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &(i64, u8)> {
        self.inner.iter()
    }
}
