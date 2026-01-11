use my_service_bus::abstractions::AsMessageId;
use my_service_bus::shared::sub_page::SubPageId;
use rust_extensions::{date_time::DateTimeAsMicroseconds, StopWatch};

use std::sync::Arc;

use crate::{
    app::AppContext,
    messages_page::MessagesPageList,
    queue_subscribers::SubscriberId,
    queues::TopicQueue,
    sessions::MyServiceBusSession,
    sub_page::GetMessageResult,
    topics::{Topic, TopicInner},
};

use super::SubscriberPackageBuilder;

pub fn try_to_deliver_to_subscribers(
    app: &AppContext,
    topic: &Arc<Topic>,
    topic_data: &mut TopicInner,
) {
    let sw = StopWatch::new();
    let mut to_send = Vec::new();

    for topic_queue in topic_data.queues.get_all_mut() {
        compile_packages(app, topic, &mut to_send, topic_queue, &mut topic_data.pages);
    }

    if to_send.len() > 0 {
        for package_builder in to_send {
            crate::operations::send_package::send_new_messages_to_deliver(
                package_builder,
                topic_data,
                sw.duration(),
            );
        }
    }
}

fn compile_packages(
    app: &AppContext,
    topic: &Arc<Topic>,
    to_send: &mut Vec<SubscriberPackageBuilder>,
    topic_queue: &mut TopicQueue,
    pages: &mut MessagesPageList,
) {
    let mut not_engaged_topics = Vec::new();

    while topic_queue.queue.queue_size() > 0 {
        let subscriber = topic_queue
            .subscribers
            .get_and_rent_next_subscriber_ready_to_deliver();

        let Some((subscriber_id, session)) = subscriber else {
            break;
        };

        let package_builder =
            compile_package(app, topic, topic_queue, pages, subscriber_id, &session);

        if let Some(package_builder) = package_builder {
            to_send.push(package_builder);
        } else {
            not_engaged_topics.push(subscriber_id);
        }
    }

    for subscriber_id in not_engaged_topics {
        topic_queue.subscribers.cancel_rent(subscriber_id);
    }
}

fn compile_package(
    app: &AppContext,
    topic: &Arc<Topic>,
    topic_queue: &mut TopicQueue,
    pages: &mut MessagesPageList,
    subscriber_id: SubscriberId,
    session: &MyServiceBusSession,
) -> Option<SubscriberPackageBuilder> {
    let mut package_builder: Option<SubscriberPackageBuilder> = None;

    #[cfg(test)]
    println!("compile_and_deliver");

    let mut payload_size = 0;

    let last_access = DateTimeAsMicroseconds::now();

    while payload_size < app.get_max_delivery_size() {
        if let Some(max_messages_per_payload) = topic_queue.max_messages_per_payload {
            if let Some(package_builder) = package_builder.as_ref() {
                if package_builder.messages_on_delivery.queue_size() >= max_messages_per_payload {
                    break;
                }
            }
        }

        let Some(message_id) = topic_queue.queue.peek() else {
            break;
        };

        let message_id = message_id.as_message_id();

        let sub_page_id: SubPageId = message_id.into();

        let sub_page = match pages.get_mut(sub_page_id) {
            Some(sub_page) => sub_page,
            None => {
                app.restore_page_scheduler
                    .schedule_load_sub_page(topic.clone(), sub_page_id);

                return package_builder;
            }
        };

        sub_page.update_last_accessed(last_access);

        topic_queue.queue.dequeue();

        match sub_page.get_message(message_id.as_message_id()) {
            GetMessageResult::Message(message_content) => {
                let attempt_no = topic_queue.delivery_attempts.get(message_content.id);

                if package_builder.is_none() {
                    package_builder = Some(SubscriberPackageBuilder::new(
                        session.clone(),
                        topic.clone(),
                        topic_queue.queue_id.clone(),
                        subscriber_id,
                    ));
                }

                package_builder
                    .as_mut()
                    .unwrap()
                    .add_message(message_content, attempt_no);
            }
            GetMessageResult::Missing => {}
            GetMessageResult::NotLoaded => {
                app.restore_page_scheduler
                    .schedule_load_sub_page(topic.clone(), sub_page_id);

                return package_builder;
            }
        }

        payload_size = match package_builder {
            Some(ref package_builder) => package_builder.get_data_size(),
            None => 0,
        };
    }

    package_builder
}

#[cfg(test)]
mod tests {

    use my_service_bus::abstractions::SbMessageHeaders;
    use my_service_bus::abstractions::{
        publisher::MessageToPublish, queue_with_intervals::QueueWithIntervals,
        subscriber::TopicQueueType,
    };
    use my_service_bus::shared::protobuf_models::MessageProtobufModel;
    use rust_extensions::date_time::DateTimeAsMicroseconds;

    #[tokio::test]
    async fn test_publish_subscribe_case() {
        const TOPIC_NAME: &str = "test-topic";
        const QUEUE_NAME: &str = "test-queue";

        let app = crate::test_tools::create_app_context().await;

        let test_session = app.sessions.add_test().await;

        crate::operations::create_topic_if_not_exists(
            &app,
            Some(test_session.session_id),
            TOPIC_NAME,
        )
        .await
        .unwrap();

        crate::operations::subscriber::subscribe_to_queue(
            &app,
            TOPIC_NAME.to_string(),
            QUEUE_NAME.to_string(),
            TopicQueueType::Permanent,
            test_session.clone().into(),
        )
        .await
        .unwrap();

        let msg1 = MessageToPublish {
            headers: SbMessageHeaders::new(),
            content: vec![0u8, 1u8, 2u8],
        };

        let msg2 = MessageToPublish {
            headers: SbMessageHeaders::new(),
            content: vec![3u8, 4u8, 5u8],
        };

        let messages = vec![msg1, msg2];

        crate::operations::publisher::publish(
            &app,
            TOPIC_NAME,
            messages,
            false,
            test_session.session_id,
        )
        .await
        .unwrap();

        app.restore_page_scheduler.emulate_event_loop_tick().await;

        let result_packets = test_session.get_list_of_packets_and_clear_them();
        assert_eq!(result_packets.len(), 1);
    }

    #[tokio::test]
    async fn test_we_subscriber_and_deliver_persisted_messages() {
        const TOPIC_NAME: &str = "test-topic";
        const QUEUE_NAME: &str = "test-queue";

        let app = crate::test_tools::create_app_context().await;
        let test_session = app.sessions.add_test().await;

        app.topic_list.add(TOPIC_NAME, 3.into(), true).await;

        //Simulate that we have persisted messages
        let msg1 = MessageProtobufModel::new(
            1.into(),
            DateTimeAsMicroseconds::now(),
            vec![0u8, 1u8, 2u8],
            vec![],
        );

        let msg2 = MessageProtobufModel::new(
            2.into(),
            DateTimeAsMicroseconds::now(),
            vec![0u8, 1u8, 2u8],
            vec![],
        );

        let messages_to_persist = vec![msg1, msg2];

        app.persistence_client
            .save_messages(TOPIC_NAME, messages_to_persist)
            .await
            .unwrap();

        {
            let topic = app.topic_list.get(TOPIC_NAME).await.unwrap();
            let mut topic_data = topic.get_access().await;

            let mut queue_with_intervals = QueueWithIntervals::new();

            queue_with_intervals.enqueue(1);
            queue_with_intervals.enqueue(2);

            topic_data.queues.restore(
                TOPIC_NAME.into(),
                QUEUE_NAME.into(),
                TopicQueueType::Permanent,
                queue_with_intervals,
            );
        }

        crate::operations::subscriber::subscribe_to_queue(
            &app,
            TOPIC_NAME.to_string(),
            QUEUE_NAME.to_string(),
            TopicQueueType::Permanent,
            test_session.clone().into(),
        )
        .await
        .unwrap();

        app.restore_page_scheduler.emulate_event_loop_tick().await;

        let mut result_packets = test_session.get_list_of_packets_and_clear_them();
        assert_eq!(result_packets.len(), 1);

        let packet = result_packets.remove(0);

        assert_eq!(TOPIC_NAME, packet.topic_id.as_str());
        assert_eq!(QUEUE_NAME, packet.queue_id.as_str());
        println!("ConfirmationId: {}", packet.subscriber_id.get_value());
    }
}
