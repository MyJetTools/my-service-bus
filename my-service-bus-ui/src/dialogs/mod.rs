use dioxus::prelude::*;

mod delete_queue;
mod delete_topic;
mod dialog_template;

pub use delete_queue::*;
pub use delete_topic::*;
pub use dialog_template::*;

#[derive(Clone)]
pub enum DialogState {
    None,
    DeleteTopic {
        topic_id: String,
        on_ok: EventHandler<String>,
    },
    DeleteQueue {
        topic_id: String,
        queue_id: String,
        on_ok: EventHandler<()>,
    },
}

#[component]
pub fn RenderDialog() -> Element {
    let state = consume_context::<Signal<DialogState>>().read().clone();
    match state {
        DialogState::None => rsx! {},
        DialogState::DeleteTopic { topic_id, on_ok } => rsx! {
            DeleteTopicDialog { topic_id, on_ok }
        },
        DialogState::DeleteQueue {
            topic_id,
            queue_id,
            on_ok,
        } => rsx! {
            DeleteQueueDialog { topic_id, queue_id, on_ok }
        },
    }
}
