mod api;
mod components;
mod dialogs;
mod models;
mod utils;
mod views;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};

use crate::views::RenderMyServiceBus;

const APP_CSS: Asset = asset!("/assets/app.css");

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting my-service-bus-ui");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    use_context_provider(|| Signal::new(crate::dialogs::DialogState::None));

    rsx! {
        document::Stylesheet { href: APP_CSS }
        RenderMyServiceBus {}
    }
}
