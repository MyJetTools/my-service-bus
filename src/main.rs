use app::AppContext;

use background::{
    DeadSubscribersKickerTimer, GcTimer, ImmediatelyPersistEventLoop, MetricsTimer,
    PersistTopicsAndQueuesTimer,
};
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
mod messages_page;
mod metric_data;
mod operations;
mod queue_subscribers;
mod queues;
mod sessions;
mod settings;
mod tcp;
mod utils;

mod background;
mod topics;
pub mod persistence_grpc {
    tonic::include_proto!("persistence");
}

#[global_allocator]
static ALLOC: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

#[tokio::main]
async fn main() {
    let settings = settings::SettingsModel::read().await;

    let app = Arc::new(AppContext::new(settings).await);

    app.immediately_persist_event_loop
        .register_event_loop(Arc::new(ImmediatelyPersistEventLoop::new(app.clone())))
        .await;

    tokio::task::spawn(crate::operations::initialization::init(app.clone()));

    let tcp_server = TcpServer::new(
        "MySbTcpServer".to_string(),
        SocketAddr::from(([0, 0, 0, 0], 6421)),
    );

    tcp_server
        .start(
            Arc::new(my_service_bus::tcp_contracts::SbTcpSerializerMetadataFactory),
            Arc::new(TcpServerEvents::new(app.clone())),
            app.states.clone(),
            my_logger::LOGGER.clone(),
        )
        .await;

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

    let mut persist_timer = MyTimer::new(app.settings.persist_timer_interval);
    persist_timer.register_timer(
        "PersistTopicsAndQueues",
        Arc::new(PersistTopicsAndQueuesTimer::new(app.clone())),
    );

    let mut gc_timer = MyTimer::new(Duration::from_secs(3));
    gc_timer.register_timer("GC", Arc::new(GcTimer::new(app.clone())));
    gc_timer.register_timer(
        "DeadSubscribers",
        Arc::new(DeadSubscribersKickerTimer::new(app.clone())),
    );

    metrics_timer.start(app.clone(), my_logger::LOGGER.clone());
    persist_timer.start(app.clone(), my_logger::LOGGER.clone());
    gc_timer.start(app.clone(), my_logger::LOGGER.clone());
    app.immediately_persist_event_loop.start(app.clone()).await;

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
