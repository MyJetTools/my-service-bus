#[cfg(test)]
use std::sync::Arc;

use super::{HttpConnectionData, TcpConnectionData};

pub enum SessionConnection {
    Tcp(TcpConnectionData),
    Http(HttpConnectionData),

    #[cfg(test)]
    Test(Arc<super::TestConnectionData>),
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

    #[cfg(test)]
    pub fn unwrap_as_test(&self) -> Arc<super::TestConnectionData> {
        if let SessionConnection::Test(data) = self {
            return data.clone();
        }

        panic!(
            "You are trying to get session as Test, but session has [{}] type",
            self.get_connection_type()
        );
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
