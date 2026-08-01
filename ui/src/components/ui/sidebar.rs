use dioxus::prelude::*;

use crate::views::my_service_bus::state::SidebarSection;

#[component]
pub fn Sidebar(
    active: SidebarSection,
    topics: usize,
    sessions: usize,
    pages: usize,
    is_live: bool,
    on_select: EventHandler<SidebarSection>,
) -> Element {
    rsx! {
        div { class: "msb-sidebar",
            div { class: "msb-sidebar__brand",
                img {
                    class: "msb-sidebar__logo",
                    src: asset!("/assets/favicon.svg"),
                    alt: "MyServiceBus",
                }
                div {
                    div { class: "msb-sidebar__name", "MyServiceBus" }
                    div { class: "msb-sidebar__sub", "admin" }
                }
            }

            div { class: "msb-sidebar__section", "Workspace" }
            div { class: "msb-sidebar__nav",
                {nav_item(SidebarSection::Topics, active, "Topics", topics, super::icon_topics(), on_select)}
                {nav_item(SidebarSection::Sessions, active, "Sessions", sessions, super::icon_sessions(), on_select)}
                {nav_item(SidebarSection::Pages, active, "Pages", pages, super::icon_pages(), on_select)}
            }

            div { class: "msb-sidebar__spacer" }

            div { class: "msb-sidebar__footer",
                div { class: "msb-sidebar__connection",
                    span {
                        class: if is_live { "msb-sidebar__pulse" } else { "msb-sidebar__pulse is-stale" },
                    }
                    span {
                        if is_live { "Connected" } else { "Disconnected" }
                    }
                }
            }
        }
    }
}

fn nav_item(
    section: SidebarSection,
    active: SidebarSection,
    label: &str,
    count: usize,
    icon: Element,
    on_select: EventHandler<SidebarSection>,
) -> Element {
    let class = if section == active {
        "msb-sidebar__item is-active"
    } else {
        "msb-sidebar__item"
    };
    let label = label.to_string();
    rsx! {
        button {
            class: "{class}",
            onclick: move |_| on_select.call(section),
            span { class: "msb-sidebar__icon", {icon} }
            span { class: "msb-sidebar__label", "{label}" }
            span { class: "msb-sidebar__count", "{count}" }
        }
    }
}
