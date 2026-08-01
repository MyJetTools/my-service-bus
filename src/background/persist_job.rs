use std::sync::Arc;

use rust_extensions::background_executor::{BackgroundJob, RepeatIteration};

use crate::app::AppContext;

pub struct PersistJob {
    app: Arc<AppContext>,
}

impl PersistJob {
    pub fn new(app: Arc<AppContext>) -> Self {
        Self { app }
    }
}

#[async_trait::async_trait]
impl BackgroundJob for PersistJob {
    async fn execute(&self) -> RepeatIteration {
        crate::operations::persist_all(&self.app).await;
        RepeatIteration::No
    }
}
