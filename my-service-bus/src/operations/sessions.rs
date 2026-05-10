use crate::{app::AppContext, sessions::MyServiceBusSession};

pub async fn disconnect(app: &AppContext, disconnected_session: MyServiceBusSession) {
    let topics = app.topic_list.get_all();

    for topic in topics.iter() {
        let mut topic_data = topic.get_access();

        let removed_subscribers = topic_data.disconnect(disconnected_session.session_id);

        if let Some(removed_subscribers) = removed_subscribers {
            for (topic_queue, removed_subscriber) in removed_subscribers {
                let name_and_version = disconnected_session.get_name_and_version();
                println!(
                    "Subscriber {} with connection_id {} is removed during the session [{}]/[{}:{:?}] disconnect process",
                    removed_subscriber.id.get_value(),
                    removed_subscriber.session.session_id.get_value(),
                    disconnected_session.session_id.get_value(),
                    name_and_version.name,
                    name_and_version.version
                );
                crate::operations::subscriber::remove_subscriber(topic_queue, removed_subscriber);
            }
        }
    }
}
