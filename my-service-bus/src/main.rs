use app::AppContext;

use background::{
    DeadSubscribersKickerTimer, GcDeletedTopicsTimer, GcTimer, MetricsTimer, PersistJob,
};
use my_tcp_sockets::{unix_socket_server::UnixSocketServer, TcpServer};
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
pub mod persistence_grpc {
    tonic::include_proto!("persistence");
}

#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
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
        SocketAddr::from(([0, 0, 0, 0], 6421)),
    );

    tcp_server
        .start(
            Arc::new(my_service_bus::tcp_contracts::MySbSerializerFactory),
            TcpServerEvents::new(app.clone()),
            app.states.clone(),
            my_logger::LOGGER.clone(),
        )
        .await;

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

    let http_connections_counter = crate::http::start_up::setup_server(&app);

    let mut metrics_timer = MyTimer::new(Duration::from_secs(1));
    metrics_timer.register_timer(
        "Metrics",
        Arc::new(MetricsTimer::new(
            app.clone(),
            http_connections_counter,
            tcp_server.threads_statistics,
        )),
    );

    let mut gc_timer = MyTimer::new(Duration::from_secs(3));
    gc_timer.register_timer("GC", Arc::new(GcTimer::new(app.clone())));
    gc_timer.register_timer(
        "DeadSubscribers",
        Arc::new(DeadSubscribersKickerTimer::new(app.clone())),
    );

    let mut gc_deleted_topics_timer = MyTimer::new(Duration::from_secs(60));
    gc_deleted_topics_timer.register_timer(
        "GcDeletedTopics",
        Arc::new(GcDeletedTopicsTimer::new(app.clone())),
    );

    metrics_timer.start(app.clone(), my_logger::LOGGER.clone());
    gc_timer.start(app.clone(), my_logger::LOGGER.clone());
    gc_deleted_topics_timer.start(app.clone(), my_logger::LOGGER.clone());
    app.persist_executor.start(my_logger::LOGGER.clone());

    #[cfg(not(test))]
    app.restore_page_scheduler
        .restore_page_events_loop
        .register_event_loop(Arc::new(crate::background::RestoreSubPagesEventLoop::new(
            app.clone(),
        )));

    #[cfg(not(test))]
    app.restore_page_scheduler
        .restore_page_events_loop
        .start(app.states.clone(), my_logger::LOGGER.clone());

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
