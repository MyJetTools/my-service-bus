use my_service_bus::shared::sub_page::SubPageId;
use rust_extensions::date_time::{AtomicDateTimeAsMicroseconds, DateTimeAsMicroseconds};

pub struct MissingSubPageInner {
    pub sub_page_id: SubPageId,
    pub created: DateTimeAsMicroseconds,
    pub last_accessed: AtomicDateTimeAsMicroseconds,
}

impl MissingSubPageInner {
    pub fn new(sub_page_id: SubPageId) -> Self {
        let created = DateTimeAsMicroseconds::now();
        Self {
            sub_page_id,
            created,
            last_accessed: AtomicDateTimeAsMicroseconds::new(created.unix_microseconds),
        }
    }

    pub fn update_last_accessed(&self, now: DateTimeAsMicroseconds) {
        self.last_accessed.update(now);
    }
}
