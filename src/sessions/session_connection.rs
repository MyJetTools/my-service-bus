#[cfg(test)]
use my_service_bus_tcp_shared::TcpContract;
#[cfg(test)]
use std::sync::Mutex;

use super::{HttpConnectionData, TcpConnectionData};
#[cfg(test)]
pub struct TestConnection {
    pub id: u8,
    pub ip: String,
    pub connected: std::sync::atomic::AtomicBool,
    pub sent_packets: Mutex<Vec<TcpContract>>,
    pub tcp_contract: Mutex<Option<TcpContract>>,
}

#[cfg(test)]
impl TestConnection {
    pub fn new(id: u8, ip: String) -> Self {
        Self {
            id,
            ip,
            connected: std::sync::atomic::AtomicBool::new(true),
            sent_packets: Mutex::new(vec![]),
            tcp_contract: Mutex::new(None),
        }
    }

    pub fn send_package(&self, tcp_contract: TcpContract) {
        let mut place_holder = self.tcp_contract.lock().unwrap();
        *place_holder = Some(tcp_contract);
    }
}

pub enum SessionConnection {
    Tcp(TcpConnectionData),
    Http(HttpConnectionData),

    #[cfg(test)]
    Test(TestConnection),
}

impl SessionConnection {
    pub fn unwrap_as_http(&self) -> &HttpConnectionData {
        if let SessionConnection::Http(data) = self {
            return data;
        }

        panic!(
            "You are trying to get session as Http type, but session has [{}] type",
            self.get_connection_type()
        );
    }

    pub fn is_http(&self) -> bool {
        match self {
            SessionConnection::Http(_) => true,
            _ => false,
        }
    }

    pub fn get_ip(&self) -> String {
        match self {
            SessionConnection::Tcp(data) => match &data.connection.addr {
                Some(addr) => addr.to_string(),
                None => "N/A".to_string(),
            },
            SessionConnection::Http(data) => data.ip.to_string(),
            #[cfg(test)]
            SessionConnection::Test(connection) => connection.ip.to_string(),
        }
    }

    pub fn get_connection_type(&self) -> &str {
        match self {
            SessionConnection::Tcp(_) => "Tcp",
            SessionConnection::Http(_) => "Http",
            #[cfg(test)]
            SessionConnection::Test(_) => "Test",
        }
    }
}
