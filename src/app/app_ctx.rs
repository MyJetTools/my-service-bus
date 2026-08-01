use std::{
    sync::{atomic::AtomicI64, Arc},
    time::Duration,
};

use rust_extensions::{date_time::DateTimeAsMicroseconds, AppStates, ApplicationStates};

use crate::{
    grpc_client::PersistenceGrpcService, namespaces::NamespacesList,
    queue_subscribers::SubscriberIdGenerator, sessions::SessionsList, settings::SettingsModel,
    utils::MultiThreadedShortString,
};

use super::prometheus_metrics::PrometheusMetrics;

pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// How long the MCP write tools stay enabled after somebody clicks "Enable" in
/// the UI. They auto-disable when it runs out.
pub const MCP_WRITES_WINDOW: Duration = Duration::from_secs(600);

pub struct AppContext {
    pub states: Arc<AppStates>,
    /// Every topic lives inside a namespace. A client which names none works in the
    /// default one, which is what everybody did before namespaces existed.
    pub namespaces: NamespacesList,
    pub persistence_client: Arc<PersistenceGrpcService>,
    pub sessions: SessionsList,
    pub subscriber_id_generator: SubscriberIdGenerator,

    pub prometheus: PrometheusMetrics,

    pub delivery_timeout: Duration,

    pub persist_executor: rust_extensions::background_executor::BackgroundExecutor,

    pub persistence_version: MultiThreadedShortString,

    #[cfg(not(test))]
    pub restore_page_scheduler: super::LoadSubPageScheduler,

    #[cfg(test)]
    pub restore_page_scheduler: crate::test_tools::SubPageLoaderSchedulerMock,

    pub debug_console: super::DebugConsole,

    pub settings: Arc<SettingsModel>,

    /// Expiry (`unix_microseconds`) of the current MCP-writes enable window.
    /// `0` means the write tools are disabled. Runtime-only — never persisted,
    /// so a restart always leaves MCP writes off.
    mcp_writes_enabled_until: AtomicI64,
}

impl AppContext {
    pub async fn new(messages_repo: PersistenceGrpcService, settings: Arc<SettingsModel>) -> Self {
        Self {
            states: Arc::new(AppStates::create_un_initialized()),
            namespaces: NamespacesList::new(),

            persistence_client: Arc::new(messages_repo),
            sessions: SessionsList::new(),

            subscriber_id_generator: SubscriberIdGenerator::new(),
            prometheus: PrometheusMetrics::new(),

            delivery_timeout: if let Some(delivery_timeout) = settings.delivery_timeout {
                delivery_timeout
            } else {
                Duration::from_secs(30)
            },
            persist_executor: rust_extensions::background_executor::BackgroundExecutor::new(
                "Persist",
            ),
            persistence_version: MultiThreadedShortString::new(),

            restore_page_scheduler: Default::default(),
            debug_console: super::DebugConsole::new(),
            settings,
            mcp_writes_enabled_until: AtomicI64::new(0),
        }
    }

    /// Opens the MCP-writes window for `MCP_WRITES_WINDOW`.
    ///
    /// Adds the window to what is LEFT of the current one instead of resetting it:
    /// the UI offers this as an "Extend" button while a window is already open,
    /// and pressing it must never be able to shorten one. `fetch_update` rather
    /// than load-then-store so two people pressing at the same moment both add
    /// their ten minutes instead of one of them being silently dropped.
    pub fn enable_mcp_writes(&self) {
        let now = DateTimeAsMicroseconds::now();

        let _ = self.mcp_writes_enabled_until.fetch_update(
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
            |current| {
                // An expired window — or one that was never opened — starts
                // counting from now; a live one keeps its remainder and grows.
                let base = if current > now.unix_microseconds {
                    DateTimeAsMicroseconds::new(current)
                } else {
                    now
                };

                Some(base.add(MCP_WRITES_WINDOW).unix_microseconds)
            },
        );
    }

    /// Closes the window immediately.
    pub fn disable_mcp_writes(&self) {
        self.mcp_writes_enabled_until
            .store(0, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_mcp_write_enabled(&self) -> bool {
        self.mcp_writes_remaining_secs().is_some()
    }

    /// Seconds left in the window, or `None` when the write tools are disabled.
    pub fn mcp_writes_remaining_secs(&self) -> Option<u64> {
        let until = self
            .mcp_writes_enabled_until
            .load(std::sync::atomic::Ordering::Relaxed);

        let now = DateTimeAsMicroseconds::now().unix_microseconds;

        if until <= now {
            return None;
        }

        Some(((until - now) / 1_000_000) as u64)
    }

    pub fn get_max_delivery_size(&self) -> usize {
        self.settings.max_delivery_size
    }

    pub fn get_default_namespace(&self) -> Arc<crate::namespaces::Namespace> {
        self.namespaces.get_default()
    }
}

impl ApplicationStates for AppContext {
    fn is_initialized(&self) -> bool {
        self.states.is_initialized()
    }

    fn is_shutting_down(&self) -> bool {
        self.states.is_shutting_down()
    }
}
