use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use my_tcp_sockets::{TcpClient, TcpClientSocketSettings, TlsSettings};

use my_service_bus::tcp_contracts::{
    MySbSerializerFactory, MySbSerializerState, MySbTcpSerializer,
};

use crate::app::AppContext;

use super::callback::MasterClientCallback;

const MASTER_CLIENT_NAME: &str = "my-service-bus-node-master-client";

struct MasterClientSettings {
    addr: String,
}

#[async_trait]
impl TcpClientSocketSettings for MasterClientSettings {
    async fn get_host_port(&self) -> Option<String> {
        Some(self.addr.clone())
    }

    async fn get_tls_settings(&self) -> Option<TlsSettings> {
        None
    }
}

pub async fn start_master_client(app: Arc<AppContext>, master_tcp_url: String) {
    let settings = Arc::new(MasterClientSettings {
        addr: master_tcp_url,
    });

    let tcp_client = TcpClient::new(MASTER_CLIENT_NAME.to_string(), settings)
        .set_seconds_to_ping(3)
        .set_disconnect_timeout(Duration::from_secs(15))
        .set_reconnect_timeout(Duration::from_secs(3));

    let callback = MasterClientCallback::new(app.clone());

    tcp_client
        .start::<_, MySbTcpSerializer, MySbSerializerState, MySbSerializerFactory, MasterClientCallback>(
            Arc::new(MySbSerializerFactory),
            callback,
            my_logger::LOGGER.clone(),
        )
        .await;
}
