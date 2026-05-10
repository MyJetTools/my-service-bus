mod api;
mod components;
mod dialogs;
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
    use_context_provider(|| Signal::new(crate::dialogs::DialogState::None));

    rsx! {
        document::Stylesheet { href: STYLED_CSS }
        document::Stylesheet { href: APP_CSS }
        style {
            "html, body, #main {{ height: 100vh; width: 100vw; margin: 0; padding: 0; overflow: hidden; }}
             .selectable {{ user-select: text; -webkit-user-select: text; -ms-user-select: text; cursor: text; }}
             .no-scrollbar {{ scrollbar-width: none; -ms-overflow-style: none; }}
             .no-scrollbar::-webkit-scrollbar {{ width: 0; height: 0; }}
             .sticky-thead thead th {{ position: sticky; top: 0; z-index: 5; }}
             .queues-header {{ display: flex; align-items: center; gap: 16px; }}
             .queues-header > span {{ flex: 0 0 auto; }}
             .header-search {{
                 margin-left: auto;
                 width: 100%; max-width: 300px;
                 height: 30px;
                 padding: 4px 10px 4px 32px;
                 background: #fff url('/assets/ico/search.svg') no-repeat 8px center / 16px;
                 color: #000; font: 13px -apple-system, sans-serif; font-weight: normal;
                 text-transform: none; letter-spacing: normal;
                 border: 1px solid #999; border-radius: 4px;
             }}
             .modal-overlay {{ position: fixed; inset: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000; }}
             .modal-card {{ background: white; padding: 24px; border-radius: 8px; min-width: 420px; box-shadow: 0 8px 30px rgba(0,0,0,0.3); }}"
        }
        RenderMyServiceBus {}
    }
}
