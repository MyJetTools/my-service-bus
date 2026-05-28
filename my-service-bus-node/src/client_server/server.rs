use std::{net::SocketAddr, sync::Arc};

use my_tcp_sockets::TcpServer;
use rust_extensions::AppStates;

use my_service_bus::tcp_contracts::MySbSerializerFactory;

use crate::app::AppContext;

use super::callback::ClientServerCallback;

const SERVER_NAME: &str = "MySbNodeClientServer";

pub async fn start_client_server(
    app: Arc<AppContext>,
    listen_tcp: String,
    app_states: Arc<AppStates>,
) {
    let addr: SocketAddr = listen_tcp
        .parse()
        .unwrap_or_else(|e| panic!("Cannot parse ListenTcp {}: {}", listen_tcp, e));

    let server = TcpServer::new(SERVER_NAME.to_string(), addr);

    server
        .start(
            Arc::new(MySbSerializerFactory),
            ClientServerCallback::new(app),
            app_states,
            my_logger::LOGGER.clone(),
        )
        .await;
}
