use my_service_bus::tcp_contracts::PacketProtVer;
use rust_extensions::date_time::DateTimeAsMicroseconds;

use super::{ConnectionMetricsSnapshot, SessionConnection, SessionId};

pub enum SessionType {
    Tcp,
    Http,
}

impl SessionType {
    pub fn as_string(&self) -> &str {
        match self {
            SessionType::Tcp => "tcp",
            SessionType::Http => "http",
        }
    }
}

pub struct SessionMetrics {
    pub name: String,
    pub version: Option<String>,
    pub ip: String,
    pub id: SessionId,
    pub connection_metrics: ConnectionMetricsSnapshot,
    pub tcp_protocol_version: Option<i32>,

    pub session_type: SessionType,
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

    pub fn get_message_to_delivery_protocol_version(&self) -> PacketProtVer {
        match &self.connection {
            SessionConnection::Tcp(data) => data.get_messages_to_deliver_protocol_version(),
            SessionConnection::Http(_) => {
                panic!("Protocol version is not applicable for HTTP Protocol")
            }
            #[cfg(test)]
            SessionConnection::Test(_) => PacketProtVer {
                tcp_protocol_version: 3.into(),
                packet_version: 0,
            },
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
