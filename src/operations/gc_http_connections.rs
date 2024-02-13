use std::time::Duration;

use crate::app::AppContext;

pub async fn gc_http_connections(app: &AppContext) {
    let inactive_session_timeout = Duration::from_secs(60);

    let disconnected_sessions = app
        .sessions
        .remove_and_disconnect_expired_http_sessions(inactive_session_timeout)
        .await;

    for disconnected_session in disconnected_sessions {
        crate::operations::sessions::disconnect(app, disconnected_session).await;
    }
}
