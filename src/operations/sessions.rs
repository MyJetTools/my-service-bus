use crate::{app::AppContext, sessions::MyServiceBusSession};

pub async fn disconnect(app: &AppContext, disconnected_session: &MyServiceBusSession) {
    let topics = app.topic_list.get_all().await;

    for topic in &topics {
        let mut topic_data = topic.get_access().await;

        let removed_subscribers = topic_data.disconnect(disconnected_session.id);

        if let Some(removed_subscribers) = removed_subscribers {
            for (topic_queue, removed_subscriber) in removed_subscribers {
                println!(
                    "Subscriber {} with connection_id {} is removed during the session [{}]/{:?} disconnect process",
                    removed_subscriber.id.get_value(),
                    removed_subscriber.session.id,
                    disconnected_session.id,
                    disconnected_session.get_name_and_client_version().await
                );
                crate::operations::subscriber::remove_subscriber(topic_queue, removed_subscriber);
            }
        }
    }
}
