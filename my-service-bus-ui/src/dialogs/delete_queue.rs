use dioxus::prelude::*;

const DIALOG_TITLE: &str = "Delete queue";

#[component]
pub fn DeleteQueueDialog(
    topic_id: String,
    queue_id: String,
    on_ok: EventHandler<()>,
) -> Element {
    let label_topic = topic_id.clone();
    let label_queue = queue_id.clone();

    let content = rsx! {
        p {
            "Confirm to delete queue "
            b { "{label_topic}/{label_queue}" }
            "?"
        }
    };

    let ok_button = rsx! {
        button {
            class: "btn btn-danger",
            onclick: move |_| {
                consume_context::<Signal<super::DialogState>>().set(super::DialogState::None);
                on_ok.call(());
            },
            "Delete"
        }
    };

    super::dialog_template(DIALOG_TITLE, content, ok_button)
}
