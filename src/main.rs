use app::AppContext;

use background::{
    DeadSubscribersKickerTimer, GcTimer, ImmediatelyPersistEventLoop, MetricsTimer,
    PersistTopicsAndQueuesTimer,
};
use my_service_bus::tcp_contracts::{ConnectionAttributes, MySbTcpSerializer};
use my_tcp_sockets::TcpServer;
use rust_extensions::MyTimer;
use tcp::socket_loop::TcpServerEvents;

use std::time::Duration;
use std::{net::SocketAddr, sync::Arc};

mod app;
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

#[tokio::main]
async fn main() {
    let settings = settings::SettingsModel::read().await;

    let app = Arc::new(AppContext::new(settings).await);

    app.immediately_persist_event_loop
        .register_event_loop(Arc::new(ImmediatelyPersistEventLoop::new(app.clone())))
        .await;

    let mut tasks = Vec::new();

    tasks.push(tokio::task::spawn(crate::operations::initialization::init(
        app.clone(),
    )));

    let tcp_server = TcpServer::new(
        "MySbTcpServer".to_string(),
        SocketAddr::from(([0, 0, 0, 0], 6421)),
    );

    tcp_server
        .start(
            Arc::new(|| -> MySbTcpSerializer {
                let attrs = ConnectionAttributes::new(0);
                MySbTcpSerializer::new(attrs)
            }),
            Arc::new(TcpServerEvents::new(app.clone())),
            app.states.clone(),
            app.logs.clone(),
        )
        .await;

    let http_connections_counter = crate::http::start_up::setup_server(&app);

    let mut metrics_timer = MyTimer::new(Duration::from_secs(1));
    metrics_timer.register_timer(
        "Metrics",
        Arc::new(MetricsTimer::new(app.clone(), http_connections_counter)),
    );

    let mut persist_and_gc_timer = MyTimer::new(app.settings.persist_timer_interval);
    persist_and_gc_timer.register_timer(
        "PersistTopicsAndQueues",
        Arc::new(PersistTopicsAndQueuesTimer::new(app.clone())),
    );
    persist_and_gc_timer.register_timer("GC", Arc::new(GcTimer::new(app.clone())));

    let mut dead_subscribers = MyTimer::new(Duration::from_secs(10));
    dead_subscribers.register_timer(
        "DeadSubscribers",
        Arc::new(DeadSubscribersKickerTimer::new(app.clone())),
    );

    metrics_timer.start(app.clone(), app.logs.clone());
    persist_and_gc_timer.start(app.clone(), app.logs.clone());
    dead_subscribers.start(app.clone(), app.logs.clone());
    app.immediately_persist_event_loop
        .start(app.clone(), app.logs.clone())
        .await;

    app.states.wait_until_shutdown().await;

    shut_down_task(app).await;

    for task in tasks {
        task.await.unwrap();
    }
}

async fn shut_down_task(app: Arc<AppContext>) {
    app.states.wait_until_shutdown().await;

    println!("Shut down detected. Waiting for 1 second to deliver all messages");
    let duration = Duration::from_secs(1);
    tokio::time::sleep(duration).await;

    crate::app::shutdown::execute(app).await;
}
