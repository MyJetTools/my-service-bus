use my_service_bus_tcp_shared::PacketProtVer;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::{ConnectionMetricsSnapshot, SessionConnection, SessionId};
pub struct SessionMetrics {
    pub name: Option<String>,
    pub version: Option<String>,
    pub ip: String,
    pub id: SessionId,
    pub connection_metrics: ConnectionMetricsSnapshot,
    pub protocol_version: String,
}

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

    pub async fn set_tcp_socket_name(
        &self,
        set_socket_name: String,
        client_version: Option<String>,
    ) {
        if let SessionConnection::Tcp(data) = &self.connection {
            data.set_socket_name(set_socket_name, client_version).await;
        } else {
            panic!("Something went wrong. You re trying to set socket name for tcp session. But session has type: {}", self.connection.get_connection_type())
        }
    }

    pub fn update_tcp_protocol_version(&self, value: i32) {
        if let SessionConnection::Tcp(connection_data) = &self.connection {
            connection_data.update_protocol_version(value);
        } else {
            panic!(
                "Invalid connection type  [{}] to update Tcp protocol version",
                self.connection.get_connection_type()
            );
        }
    }

    pub fn update_tcp_delivery_packet_version(&self, value: i32) {
        if let SessionConnection::Tcp(connection_data) = &self.connection {
            connection_data.update_deliver_message_packet_version(value);
        } else {
            panic!(
                "Invalid connection type  [{}] to update Tcp delivery packet version",
                self.connection.get_connection_type()
            );
        }
    }

    pub async fn get_name_and_client_version(&self) -> (Option<String>, Option<String>) {
        match &self.connection {
            SessionConnection::Tcp(data) => {
                let attr = data.get_attrs().await;
                (attr.name, attr.version)
            }
            SessionConnection::Http(data) => {
                (Some(data.name.to_string()), Some(data.version.to_string()))
            }
            #[cfg(test)]
            SessionConnection::Test(data) => (data.name.clone(), data.version.clone()),
        }
    }

    fn protocol_version_as_string(&self) -> String {
        match &self.connection {
            SessionConnection::Tcp(data) => format!("Tcp: {}", data.get_protocol_version()),
            SessionConnection::Http(_) => "Http".to_string(),
            #[cfg(test)]
            SessionConnection::Test(_) => "Test".to_string(),
        }
    }

    pub fn get_message_to_delivery_protocol_version(&self) -> PacketProtVer {
        match &self.connection {
            SessionConnection::Tcp(data) => data.get_messages_to_deliver_protocol_version(),
            SessionConnection::Http(_) => {
                panic!("Protocol version is not applicable for HTTP Protocol")
            }
            #[cfg(test)]
            SessionConnection::Test(_) => PacketProtVer {
                protocol_version: 3,
                packet_version: 0,
            },
        }
    }

    pub async fn get_metrics(&self) -> SessionMetrics {
        let connection_metrics = match &self.connection {
            SessionConnection::Tcp(data) => data.get_connection_metrics(),
            SessionConnection::Http(data) => data.get_connection_metrics(),
            #[cfg(test)]
            SessionConnection::Test(_) => {
                panic!("We do not have metrics in test environment");
            }
        };

        let protocol_version = self.protocol_version_as_string();

        let (name, version) = self.get_name_and_client_version().await;

        SessionMetrics {
            id: self.id,
            name,
            version,
            ip: self.connection.get_ip().to_string(),
            protocol_version,
            connection_metrics,
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
            SessionConnection::Test(connection) => {
                let result = connection
                    .connected
                    .load(std::sync::atomic::Ordering::SeqCst);

                if result == false {
                    return false;
                }

                connection
                    .connected
                    .store(false, std::sync::atomic::Ordering::SeqCst);

                return true;
            }
        }
    }
}
