use std::{net::SocketAddr, sync::Arc, time::Duration};

use tokio::{
    io::{self, AsyncWriteExt, ReadHalf},
    net::{TcpListener, TcpStream},
};

use crate::{
    app::AppContext,
    date_time::MyDateTime,
    sessions::MyServiceBusSession,
    tcp_contracts::tcp_contract::{ConnectionAttributes, TcpContract},
};

use super::{MySbSocketError, SocketReader};

pub type ConnectionId = i64;

pub async fn start(addr: SocketAddr, app: Arc<AppContext>) {
    while !app.states.is_initialized() {
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    let listener = TcpListener::bind(addr).await.unwrap();

    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::TcpSocket,
            "Tcp socket is started".to_string(),
            format!("{:?}", addr),
        )
        .await;

    let mut socket_id: ConnectionId = 0;

    while !app.states.is_shutting_down() {
        let (tcp_stream, addr) = listener.accept().await.unwrap();

        let (read_socket, mut write_socket) = io::split(tcp_stream);

        if app.states.is_shutting_down() {
            write_socket.shutdown().await.unwrap();
            break;
        }

        socket_id += 1;

        let my_sb_session = Arc::new(MyServiceBusSession::new(
            socket_id,
            format! {"{}", addr},
            write_socket,
            app.logs.clone(),
        ));

        app.sessions.add(my_sb_session.clone()).await;

        app.logs
            .add_info(
                None,
                crate::app::logs::SystemProcess::TcpSocket,
                "Accept sockets loop".to_string(),
                format!("Connected socket {}. IP: {}", socket_id, addr),
            )
            .await;

        tokio::task::spawn(process_socket(read_socket, app.clone(), my_sb_session));
    }

    app.logs
        .add_info(
            None,
            crate::app::logs::SystemProcess::TcpSocket,
            "Tcp socket is stopped".to_string(),
            format!("{:?}", addr),
        )
        .await;
}

async fn process_socket(
    read_socket: ReadHalf<TcpStream>,
    app: Arc<AppContext>,
    my_sb_session: Arc<MyServiceBusSession>,
) {
    let socket_result = socket_loop(read_socket, app.clone(), my_sb_session.clone()).await;

    if let Err(err) = socket_result {
        app.logs
            .add_error(
                None,
                crate::app::logs::SystemProcess::TcpSocket,
                format!("Socket {} Processing", my_sb_session.get_name().await),
                "Socket disconnected".to_string(),
                Some(format!("{:?}", err)),
            )
            .await;
    } else {
        app.logs
            .add_info(
                None,
                crate::app::logs::SystemProcess::TcpSocket,
                "tcp_socket_process".to_string(),
                "Socket disconnected".to_string(),
            )
            .await;
    }

    crate::operations::sessions::disconnect(app.as_ref(), my_sb_session.id).await;
}

async fn socket_loop(
    read_socket: ReadHalf<TcpStream>,
    app: Arc<AppContext>,
    session: Arc<MyServiceBusSession>,
) -> Result<(), MySbSocketError> {
    let mut socket_reader = SocketReader::new(read_socket);

    let mut attr = ConnectionAttributes::new();

    loop {
        socket_reader.start_calculating_read_size();
        let deserialize_result = TcpContract::deserialize(&mut socket_reader, &attr).await;
        session.increase_read_size(socket_reader.read_size).await;

        let now = MyDateTime::utc_now();
        session.last_incoming_package.update(now);

        match deserialize_result {
            Ok(tcp_contract) => {
                let packet_name = tcp_contract.to_string();
                let result = super::connection::handle_incoming_payload(
                    app.clone(),
                    tcp_contract,
                    session.as_ref(),
                    &mut attr,
                )
                .await;

                if let Err(err) = result {
                    session
                        .send(TcpContract::Reject {
                            message: format!("Handling message {}: {:?}", packet_name, err),
                        })
                        .await;
                }
            }
            Err(err) => return Err(err),
        }
    }
}
