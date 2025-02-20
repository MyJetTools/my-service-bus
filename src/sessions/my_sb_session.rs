use my_service_bus::tcp_contracts::PacketProtVer;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::operations::delivery::SubscriberPackageBuilder;

use super::{ConnectionMetricsSnapshot, SessionId};

pub enum SessionType {
    Tcp(PacketProtVer),
    Http,
    #[cfg(test)]
    Test,
}

impl SessionType {
    pub fn as_str(&self) -> &str {
        match self {
            SessionType::Tcp(_) => "tcp",
            SessionType::Http => "http",
            #[cfg(test)]
            SessionType::Test => "test",
        }
    }
}

pub struct SessionMetrics {
    pub ip: String,
    pub connection_metrics: ConnectionMetricsSnapshot,
    pub connected: DateTimeAsMicroseconds,
    pub tcp_protocol_version: Option<i32>,
}

pub struct SessionNameAndVersion {
    pub name: String,
    pub version: Option<String>,
    pub env_info: Option<String>,
}

#[async_trait::async_trait]
pub trait MyServiceBusSession {
    fn get_session_id(&self) -> SessionId;
    fn get_name_and_version(&self) -> SessionNameAndVersion;
    fn get_session_type(&self) -> SessionType;

    fn get_metrics(&self) -> SessionMetrics;

    async fn disconnect(&self) -> bool;

    async fn send_messages_to_connection(&self, package_builder: SubscriberPackageBuilder);
}

/*
pub struct MyServiceBusSession {
    pub id: SessionId,
    pub connection: SessionConnection,
    pub connected: DateTimeAsMicroseconds,
}

impl MyServiceBusSession {
    pub fn new(id: SessionId, connection: SessionConnection) -> Self {
        Self {
            connection,
            id,
            connected: DateTimeAsMicroseconds::now(),
        }
    }

    pub fn update_tcp_delivery_packet_version(&self, value: u8) {
        if let SessionConnection::Tcp(connection_data) = &self.connection {
            connection_data.update_deliver_message_packet_version(value);
        } else {
            panic!(
                "Invalid connection type  [{}] to update Tcp delivery packet version",
                self.connection.get_connection_type()
            );
        }
    }

    pub fn get_name_and_client_version(&self) -> (String, Option<String>) {
        match &self.connection {
            SessionConnection::Tcp(data) => (data.name.to_string(), data.version.clone()),
            SessionConnection::Http(data) => {
                (data.name.to_string(), Some(data.version.to_string()))
            }
            #[cfg(test)]
            SessionConnection::Test(data) => (data.name.clone(), data.version.clone()),
        }
    }

    fn get_tcp_protocol_version(&self) -> Option<i32> {
        match &self.connection {
            SessionConnection::Tcp(data) => data.get_protocol_version().into(),
            SessionConnection::Http(_) => None,
            #[cfg(test)]
            SessionConnection::Test(_) => None,
        }
    }

    pub async fn get_metrics(&self) -> SessionMetrics {
        let (connection_metrics, session_type) = match &self.connection {
            SessionConnection::Tcp(data) => (data.get_connection_metrics(), SessionType::Tcp),
            SessionConnection::Http(data) => (data.get_connection_metrics(), SessionType::Http),
            #[cfg(test)]
            SessionConnection::Test(_) => {
                panic!("We do not have metrics in test environment");
            }
        };

        let tcp_protocol_version = self.get_tcp_protocol_version();

        let (name, version) = self.get_name_and_client_version();

        SessionMetrics {
            id: self.id,
            name,
            version,
            ip: self.connection.get_ip().to_string(),
            tcp_protocol_version,
            connection_metrics,
            session_type,
        }
    }

    pub async fn deliver_messages(mut package_builder: SubscriberPackageBuilder) {
        let session = package_builder.session.clone();
        match &session.connection {
            crate::sessions::SessionConnection::Tcp(data) => {
                let tcp_contract = package_builder.get_tcp_result();
                let connection = data.connection.clone();

                connection.send(&tcp_contract).await;
            }
            crate::sessions::SessionConnection::Http(data) => {
                let messages = package_builder.get_http_result();

                data.send_messages(
                    package_builder.topic.topic_id.clone(),
                    package_builder.queue_id.clone(),
                    package_builder.subscriber_id.clone(),
                    messages,
                )
                .await;
            }
            #[cfg(test)]
            crate::sessions::SessionConnection::Test(data) => {
                let messages = package_builder.get_http_result();

                data.send_messages(
                    package_builder.topic.topic_id.clone(),
                    package_builder.queue_id.clone(),
                    package_builder.subscriber_id.clone(),
                    messages,
                )
                .await;
            }
        }

        /*
        tokio::spawn(async move {
            match &session.connection {
                crate::sessions::SessionConnection::Tcp(data) => {
                    let send_new_messages = package_builder.unwrap_tcp_result();
                    data.connection.send(&send_new_messages.tcp_contract).await;
                }
                crate::sessions::SessionConnection::Http(data) => {
                    data.send_packet(tcp_packet).await;
                }
                #[cfg(test)]
                crate::sessions::SessionConnection::Test(data) => {
                    data.send_packet(tcp_packet).await;
                }
            }
        });
         */
    }

    #[cfg(test)]
    pub fn is_disconnected(&self) -> bool {
        match &self.connection {
            SessionConnection::Tcp(itm) => itm.connection.is_connected(),
            SessionConnection::Http(itm) => itm.is_connected(),
            SessionConnection::Test(itm) => itm.is_connected(),
        }
    }

    pub async fn disconnect(&self) -> bool {
        match &self.connection {
            SessionConnection::Tcp(data) => {
                return data.connection.disconnect().await;
            }
            SessionConnection::Http(data) => {
                return data.disconnect();
            }
            #[cfg(test)]
            SessionConnection::Test(connection) => return connection.disconnect(),
        }
    }



    pub fn create_delivery_builder(
        session: &Arc<Self>,
        topic: Arc<Topic>,
        queue_id: QueueId,
        subscriber_id: SubscriberId,
    ) -> SubscriberPackageBuilder {
        match &session.connection {
            SessionConnection::Tcp(data) => SubscriberPackageBuilder::create_tcp(
                topic,
                queue_id,
                subscriber_id,
                session.clone(),
                data.get_messages_to_deliver_protocol_version(),
            ),
            SessionConnection::Http(_) => SubscriberPackageBuilder::create_http(
                topic,
                queue_id,
                subscriber_id,
                session.clone(),
            ),
            #[cfg(test)]
            SessionConnection::Test(_) => SubscriberPackageBuilder::create_http(
                topic,
                queue_id,
                subscriber_id,
                session.clone(),
            ),
        }
    }
}
 */
