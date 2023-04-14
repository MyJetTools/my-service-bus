use rust_extensions::ShortString;
use tokio::sync::RwLock;

pub struct MultiThreadedShortString {
    value: RwLock<ShortString>,
}

impl MultiThreadedShortString {
    pub fn new() -> Self {
        Self {
            value: RwLock::new(ShortString::new_empty()),
        }
    }

    pub async fn update(&self, new_value: &str) {
        let mut write_access = self.value.write().await;
        write_access.update(new_value);
    }

    pub async fn get(&self) -> String {
        let read_access = self.value.read().await;
        read_access.as_str().to_string()
    }
}
