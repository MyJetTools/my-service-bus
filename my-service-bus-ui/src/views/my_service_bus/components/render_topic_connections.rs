use dioxus::prelude::*;

use crate::models::*;

pub fn render_topic_connections<'a>(
    data: &'a MySbHttpContract,
    topic: &'a TopicHttpModel,
) -> impl Iterator<Item = Element> + 'a {
    let to_render = topic.publishers.iter().map(|publisher| {
        let let_class_name = if publisher.active == 0 {
            "led-gray"
        } else {
            "led-green"
        };

        let session = data.get_session(publisher.session_id);

        let render_session = match session {
            Some(session) => {
                let env_info = if let Some(env_info) = session.env_info.as_ref() {
                    rsx! {
                        span {
                            style: "background: white;color: black;",
                            class: "badge badge-light",
                            "{env_info}"
                        }
                    }
                } else {
                    rsx! {}
                };

                rsx! {
                    div {
                        b { {session.name.as_str()} }
                        div { {session.get_session_as_string()} }
                        div { {env_info} }
                        div { {session.ip.as_str()} }
                    }

                }
            }
            None => rsx! { "Not found" },
        };

        rsx! {
            div { class: "topic-connection",

                div { class: "left-part",
                    div { class: let_class_name }
                    div {
                        span { class: "badge text-bg-light", {publisher.session_id.to_string()} }
                    }
                    div {}
                }
                div { {render_session} }
            }
        }
    });

    /*
       let to_render = connections.into_iter().map(|itm| {
           rsx! {
               div { class: "topic-connection",

                   div {
                   }
                   div { {itm.name.as_str()} }
               }

           }
       });
    */
    to_render
}
