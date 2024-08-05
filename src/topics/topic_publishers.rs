use crate::sessions::SessionId;

pub struct TopicPublishers {
    inner: Vec<(SessionId, u8)>,
}

impl TopicPublishers {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }
    pub fn add(&mut self, session_id: SessionId, value: u8) {
        for (my_session_id, my_value) in &mut self.inner {
            if my_session_id.get_value() == session_id.get_value() {
                *my_value = value;
                return;
            }
        }

        self.inner.push((session_id, value));
    }

    pub fn has_session(&self, session_id: SessionId) -> bool {
        self.inner
            .iter()
            .any(|x| x.0.get_value() == session_id.get_value())
    }

    pub fn one_second_tick(&mut self) {
        for value in &mut self.inner {
            if value.1 > 0 {
                value.1 -= 1;
            }
        }
    }

    pub fn remove(&mut self, session_id: SessionId) {
        self.inner
            .retain(|x| x.0.get_value() != session_id.get_value());
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &(SessionId, u8)> {
        self.inner.iter()
    }
}
