use std::time::Duration;

use dioxus::prelude::*;
use dioxus_utils::{js::sleep, DataState, RenderState};

use crate::{
    api,
    models::{
        QueuesJsonResult, SessionJsonResult, StatusJsonResult, TopicJsonContract,
    },
};

const REFRESH_INTERVAL: Duration = Duration::from_secs(1);

#[component]
pub fn Dashboard() -> Element {
    let mut state = use_signal(DataState::<StatusJsonResult>::new);

    // Start a single background poll loop on mount. Re-renders driven by
    // `state.write()` won't restart it.
    use_hook(|| {
        spawn(async move {
            loop {
                match api::get_status().await {
                    Ok(data) => state.write().set_value(data),
                    Err(err) => state.write().set_error(err),
                }
                sleep(REFRESH_INTERVAL).await;
            }
        });
    });

    let state_ra = state.read();

    match state_ra.as_ref() {
        RenderState::None | RenderState::Loading => {
            rsx! { div { class: "loading", "Loading status..." } }
        }
        RenderState::Error(err) => rsx! {
            div { class: "error",
                h3 { "Failed to load status" }
                pre { "{err}" }
            }
        },
        RenderState::Loaded(status) => render_dashboard(status),
    }
}

fn render_dashboard(status: &StatusJsonResult) -> Element {
    rsx! {
        div { class: "dashboard",
            h2 { "MyServiceBus" }
            div { class: "versions",
                span { "version: " b { "{status.version}" } }
                span { "persistence: " b { "{status.persistence_version}" } }
                span { "memory: " b { "{status.system.usedmem} / {status.system.totalmem}" } }
            }
            {render_topics(&status.topics.items, &status.queues)}
            {render_sessions(&status.sessions.items)}
        }
    }
}

fn render_topics(
    topics: &[TopicJsonContract],
    queues: &std::collections::BTreeMap<String, QueuesJsonResult>,
) -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-header",
                span { class: "label", "Topics" }
                span { class: "count", "{topics.len()}" }
            }
            table {
                thead {
                    tr {
                        th { "Id" }
                        th { class: "num", "Message Id" }
                        th { class: "num", "Msg/sec" }
                        th { class: "num", "Pkt/sec" }
                        th { class: "num", "Avg size" }
                        th { class: "num", "Persist size" }
                        th { "Persist" }
                        th { class: "num", "Queues" }
                    }
                }
                tbody {
                    for topic in topics {
                        {render_topic_row(topic, queues)}
                    }
                }
            }
        }
    }
}

fn render_topic_row(
    topic: &TopicJsonContract,
    queues: &std::collections::BTreeMap<String, QueuesJsonResult>,
) -> Element {
    let q_count = queues.get(&topic.id).map(|q| q.queues.len()).unwrap_or(0);

    rsx! {
        tr {
            td { "{topic.id}" }
            td { class: "num", "{topic.message_id}" }
            td { class: "num", "{topic.messages_per_second}" }
            td { class: "num", "{topic.packets_per_second}" }
            td { class: "num", "{topic.mean_message_size}" }
            td { class: "num", "{topic.persist_size}" }
            td { if topic.persist { "yes" } else { "no" } }
            td { class: "num", "{q_count}" }
        }
    }
}

fn render_sessions(sessions: &[SessionJsonResult]) -> Element {
    rsx! {
        section { class: "section",
            div { class: "section-header",
                span { class: "label", "Sessions" }
                span { class: "count", "{sessions.len()}" }
            }
            table {
                thead {
                    tr {
                        th { class: "num", "Id" }
                        th { "Name" }
                        th { "Type" }
                        th { "IP" }
                        th { "Version" }
                        th { "Connected" }
                        th { "Last incoming" }
                        th { class: "num", "Read/sec" }
                        th { class: "num", "Written/sec" }
                    }
                }
                tbody {
                    for s in sessions {
                        tr {
                            td { class: "num", "{s.id}" }
                            td { "{s.name}" }
                            td { "{s.session_type}" }
                            td { class: "id-string", "{s.ip}" }
                            td { class: "id-string",
                                {s.version.clone().unwrap_or_default()}
                            }
                            td { "{s.connected}" }
                            td { "{s.last_incoming}" }
                            td { class: "num", "{s.read_per_sec}" }
                            td { class: "num", "{s.written_per_sec}" }
                        }
                    }
                }
            }
        }
    }
}
