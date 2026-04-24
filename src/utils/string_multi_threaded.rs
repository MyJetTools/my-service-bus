use std::sync::Arc;

use arc_swap::ArcSwap;
use rust_extensions::ShortString;

pub struct MultiThreadedShortString {
    value: ArcSwap<ShortString>,
}

impl MultiThreadedShortString {
    pub fn new() -> Self {
        Self {
            value: ArcSwap::from_pointee(ShortString::new_empty()),
        }
    }

    pub fn update(&self, new_value: &str) {
        let mut next = ShortString::new_empty();
        next.update(new_value);
        self.value.store(Arc::new(next));
    }

    pub fn get(&self) -> String {
        self.value.load().as_str().to_string()
    }
}
