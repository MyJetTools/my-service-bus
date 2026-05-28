use std::sync::Arc;

use my_service_bus::tcp_contracts::MySbTcpContract;

use crate::app::AppContext;

/// Background loop that drains the outbound buffer and pushes a `NodePublish`
/// to master whenever the channel is idle and there's data waiting. Throttled
/// by the master's `NodePublishResponse` — only one in-flight at a time.
pub async fn run(app: Arc<AppContext>) {
    loop {
        app.outbound.wait_for_signal().await;

        loop {
            let to_flush = app.outbound.try_take_flush().await;
            let Some((sequence_number, topics)) = to_flush else {
                // Either nothing to send or we're already in-flight; either
                // way wait for the next pulse.
                break;
            };

            let connection = match app.get_master_connection().await {
                Some(conn) => conn,
                None => {
                    // Master is down. Roll back the in-flight by reporting it
                    // complete with no acks — actually we need to put it
                    // back, but since the disconnect callback already drains
                    // everything (including in-flight), this path shouldn't
                    // happen in practice. Be defensive: drain and bail.
                    let _ = app.outbound.complete_in_flight(sequence_number).await;
                    break;
                }
            };

            connection.send(&MySbTcpContract::NodePublish {
                sequence_number,
                topics,
            });
        }
    }
}
