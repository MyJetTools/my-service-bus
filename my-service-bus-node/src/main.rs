// File-per-struct layout with `mod.rs` re-exports is the project-wide pattern;
// clippy's module_inception warning fights it for every leaf module.
#![allow(clippy::module_inception)]

mod app;
mod client_server;
mod client_sessions;
mod flusher;
mod master_client;
mod outbound;
mod settings;

use std::sync::Arc;

use rust_extensions::AppStates;

use crate::{
    app::AppContext, client_server::start_client_server, master_client::start_master_client,
    settings::SettingsModel,
};

#[tokio::main]
async fn main() {
    let settings = SettingsModel::load();
    println!(
        "Starting my-service-bus-node: listen={} master={}",
        settings.listen_tcp, settings.master_tcp_url
    );

    let app = AppContext::new();
    let app_states = Arc::new(AppStates::create_initialized());

    // Master client (background task; reconnects forever).
    tokio::spawn(start_master_client(
        app.clone(),
        settings.master_tcp_url.clone(),
    ));

    // Flusher: drains outbound buffer into NodePublish packets.
    tokio::spawn(flusher::run(app.clone()));

    // Client server: blocks the main task until shutdown.
    start_client_server(app, settings.listen_tcp, app_states).await;
}
