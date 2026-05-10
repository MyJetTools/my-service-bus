mod api;
mod components;
mod models;
mod utils;
mod views;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};

use crate::views::RenderMyServiceBus;

const STYLED_CSS: Asset = asset!("/assets/styled.css");
const APP_CSS: Asset = asset!("/assets/app.css");

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("starting my-service-bus-ui");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: STYLED_CSS }
        document::Stylesheet { href: APP_CSS }
        style { ":root {{ --left-panel-width: 0px; }} #main {{ height: 100vh; }} body {{ margin: 0; }}" }
        div { id: "main-panel",
            RenderMyServiceBus {}
        }
    }
}
