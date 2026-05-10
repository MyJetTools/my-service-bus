use dioxus::prelude::*;

use crate::views::my_service_bus::state::SidebarSection;

#[component]
pub fn Topbar(
    section: SidebarSection,
    filter: String,
    on_filter_change: EventHandler<String>,
) -> Element {
    let crumb_label = match section {
        SidebarSection::Topics => "Topics",
        SidebarSection::Sessions => "Sessions",
        SidebarSection::Pages => "Pages",
    };
    rsx! {
        div { class: "msb-topbar",
            div { class: "msb-crumbs",
                strong { "MyServiceBus" }
                span { class: "sep", "/" }
                span { "{crumb_label}" }
            }
            div { class: "msb-search",
                span { class: "msb-search__icon", {super::icon_search()} }
                input {
                    r#type: "text",
                    placeholder: "Filter topic / queue / session…",
                    value: "{filter}",
                    oninput: move |e| on_filter_change.call(e.value().to_lowercase()),
                }
                span { class: "msb-search__kbd", "⌘K" }
            }
        }
    }
}
