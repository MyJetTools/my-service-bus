use app::AppContext;

use background::{
    DeadSubscribersKickerTimer, GcDeletedTopicsTimer, GcTimer, MetricsTimer, PersistJob,
};
#[cfg(unix)]
use my_tcp_sockets::unix_socket_server::UnixSocketServer;
use my_tcp_sockets::TcpServer;
use rust_extensions::MyTimer;
use tcp::socket_events::TcpServerEvents;

use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};

mod app;
mod avg_value;
mod errors;
mod grpc_client;
mod http;
mod mappers;
mod mcp;
mod messages_page;
mod metric_data;
mod operations;
mod queue_subscribers;
mod queues;
mod sessions;
mod settings;
mod sub_page;
mod tcp;
#[cfg(test)]
mod test_tools;
mod utils;

mod background;
mod namespaces;
mod topics;
#[cfg(windows)]
mod win_service;
pub mod persistence_grpc {
    tonic::include_proto!("persistence");
}

#[cfg(target_os = "linux")]
#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

// Read by jemalloc before `main`: a background thread purges freed pages back to the OS
// after ~1s (dirty) and immediately past the MADV_FREE stage (muzzy), even when the node is idle.
// tikv-jemalloc-sys builds prefixed by default, hence `_rjem_malloc_conf`.
#[cfg(target_os = "linux")]
#[allow(non_upper_case_globals)]
#[export_name = "_rjem_malloc_conf"]
pub static malloc_conf: Option<&'static u8> =
    Some(&b"background_thread:true,dirty_decay_ms:1000,muzzy_decay_ms:0\0"[0]);

/// Linux and macOS: the node in the console, until SIGTERM or Ctrl+C.
#[cfg(not(windows))]
#[tokio::main]
async fn main() {
    run_node(|_| {}).await;
}

/// Windows: the console run, and the node as a Windows service - see `win_service`.
#[cfg(windows)]
fn main() {
    win_service::main();
}

/// The whole life of the node: it reads its settings, comes up, calls `on_started` once every
/// listener and timer runs, waits for the shutdown (a signal, or `AppStates::set_shutting_down`),
/// and hands what it holds over to the persistence before it returns. One function, so that what
/// the node runs on - the TCP server, the timers - lives exactly as long as the node.
pub async fn run_node(on_started: impl FnOnce(Arc<AppContext>)) {
    let settings = settings::SettingsModel::read().await;
    let settings = Arc::new(settings);

    let messages_repo =
        crate::grpc_client::PersistenceGrpcService::create_production_instance(settings.clone());

    let app = Arc::new(AppContext::new(messages_repo, settings).await);

    app.persist_executor
        .register(Arc::new(PersistJob::new(app.clone())));

    tokio::task::spawn(crate::operations::initialization::init(app.clone()));

    let tcp_server = TcpServer::new(
        "MySbTcpServer".to_string(),
        SocketAddr::new(app.settings.listen_ip, 6421),
    );

    tcp_server
        .start(
            Arc::new(my_service_bus::tcp_contracts::MySbSerializerFactory),
            TcpServerEvents::new(app.clone()),
            app.states.clone(),
            my_logger::LOGGER.clone(),
        )
        .await;

    #[cfg(unix)]
    let _unix_socket = if let Some(unix_socket_addr) = app.settings.listen_unix_socket.as_ref() {
        let unix_socket_addr = rust_extensions::file_utils::format_path(unix_socket_addr);
        let unix_socket = UnixSocketServer::new("MySbTcpServerUnixSocket", unix_socket_addr);
        unix_socket
            .start(
                Arc::new(my_service_bus::tcp_contracts::MySbSerializerFactory),
                TcpServerEvents::new(app.clone()),
                app.states.clone(),
                my_logger::LOGGER.clone(),
            )
            .await;

        Some(unix_socket)
    } else {
        None
    };

    // There are no unix sockets to listen on here: a setting which asks for one is said out loud
    // rather than ignored, and the TCP port serves.
    #[cfg(not(unix))]
    if let Some(unix_socket_addr) = app.settings.listen_unix_socket.as_ref() {
        println!(
            "listen_unix_socket '{unix_socket_addr}' is ignored: unix sockets are not supported on this platform"
        );
    }

    let http_connections_counter = crate::http::start_up::setup_server(&app);

    let mut metrics_timer = MyTimer::new(Duration::from_secs(1), my_logger::LOGGER.clone());
    metrics_timer.register_timer(
        "Metrics",
        Arc::new(MetricsTimer::new(
            app.clone(),
            http_connections_counter,
            tcp_server.threads_statistics,
        )),
    );

    let mut gc_timer = MyTimer::new(Duration::from_secs(3), my_logger::LOGGER.clone());
    gc_timer.register_timer("GC", Arc::new(GcTimer::new(app.clone())));
    gc_timer.register_timer(
        "DeadSubscribers",
        Arc::new(DeadSubscribersKickerTimer::new(app.clone())),
    );

    let mut gc_deleted_topics_timer =
        MyTimer::new(Duration::from_secs(60), my_logger::LOGGER.clone());
    gc_deleted_topics_timer.register_timer(
        "GcDeletedTopics",
        Arc::new(GcDeletedTopicsTimer::new(app.clone())),
    );

    metrics_timer.start();
    gc_timer.start();
    gc_deleted_topics_timer.start();
    app.persist_executor.start();

    #[cfg(not(test))]
    app.restore_page_scheduler
        .restore_page_events_loop
        .register_event_loop(Arc::new(crate::background::RestoreSubPagesEventLoop::new(
            app.clone(),
        )));

    #[cfg(not(test))]
    app.restore_page_scheduler.restore_page_events_loop.start();

    on_started(app.clone());

    app.states.wait_until_shutdown().await;

    shut_down_task(app).await;
}

async fn shut_down_task(app: Arc<AppContext>) {
    app.states.wait_until_shutdown().await;

    println!("Shut down detected. Waiting for 1 second to deliver all messages");
    let duration = Duration::from_secs(1);
    tokio::time::sleep(duration).await;

    crate::app::shutdown::execute(app).await;
}
