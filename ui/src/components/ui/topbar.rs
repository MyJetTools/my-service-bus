use dioxus::prelude::*;

use crate::models::NamespaceApiModel;
use crate::views::my_service_bus::state::SidebarSection;

#[component]
pub fn Topbar(
    section: SidebarSection,
    filter: String,
    namespaces: Vec<NamespaceApiModel>,
    /// Empty string means the default namespace — the UI then stores nothing and
    /// sends no `ns` header at all, which is what a pre-namespace client does.
    selected_namespace: String,
    on_filter_change: EventHandler<String>,
    on_namespace_change: EventHandler<String>,
) -> Element {
    let crumb_label = match section {
        SidebarSection::Topics => "Topics",
        SidebarSection::Sessions => "Sessions",
        SidebarSection::Pages => "Pages",
    };

    // The selector stays hidden until the node actually holds more than one
    // namespace: on a single-namespace broker it would be noise.
    let namespace_selector = if namespaces.len() > 1 {
        let options = namespaces.into_iter().map(|namespace| {
            // The default namespace is offered with an empty value, so picking it
            // stores nothing and the UI goes back to sending no header at all.
            let value = if namespace.name == crate::models::DEFAULT_NAMESPACE {
                String::new()
            } else {
                namespace.name.clone()
            };
            let selected = value == selected_namespace;
            rsx! {
                option { value: "{value}", selected, "{namespace.name} ({namespace.topics_amount})" }
            }
        });

        rsx! {
            div { class: "msb-ns",
                span { class: "msb-ns__label", "ns" }
                select {
                    class: "msb-ns__select",
                    value: "{selected_namespace}",
                    onchange: move |evt| on_namespace_change.call(evt.value()),
                    {options}
                }
            }
        }
    } else {
        rsx! {}
    };

    rsx! {
        div { class: "msb-topbar",
            div { class: "msb-crumbs",
                strong { "MyServiceBus" }
                span { class: "sep", "/" }
                span { "{crumb_label}" }
            }
            {namespace_selector}
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
