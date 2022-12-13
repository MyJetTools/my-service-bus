use my_service_bus_shared::sub_page::{SubPage, SubPageId};
use std::sync::Arc;

use crate::{
    app::AppContext,
    queues::TopicQueue,
    topics::{Topic, TopicData},
};

use super::SubscriberPackageBuilder;

pub fn start_new(app: &Arc<AppContext>, topic: &Arc<Topic>, topic_data: &mut TopicData) {
    while let Some(package_builder) = build_new_package_builder(topic, topic_data) {
        compile_and_deliver(app, package_builder, topic, topic_data);
    }
}

fn build_new_package_builder(
    topic: &Arc<Topic>,
    topic_data: &mut TopicData,
) -> Option<SubscriberPackageBuilder> {
    for topic_queue in topic_data.queues.get_all_mut() {
        if topic_queue.queue.len() == 0 {
            continue;
        }

        let subscriber = topic_queue
            .subscribers
            .get_and_rent_next_subscriber_ready_to_deliver();

        if subscriber.is_none() {
            continue;
        }

        let subscriber = subscriber.unwrap();

        let result = SubscriberPackageBuilder::new(
            topic.clone(),
            topic_queue.queue_id.to_string(),
            subscriber.session.clone(),
            subscriber.id,
            subscriber
                .session
                .get_message_to_delivery_protocol_version(),
        );

        return Some(result);
    }

    None
}

fn compile_and_deliver(
    app: &Arc<AppContext>,
    mut package_builder: SubscriberPackageBuilder,
    topic: &Arc<Topic>,
    topic_data: &mut TopicData,
) {
    #[cfg(test)]
    println!("compile_and_deliver");

    if let Some(topic_queue) = topic_data.queues.get_mut(package_builder.queue_id.as_ref()) {
        loop {
            let message_id = topic_queue.queue.peek();

            if message_id.is_none() {
                return;
            }

            let message_id = message_id.unwrap();

            let sub_page_id = SubPageId::from_message_id(message_id);

            let sub_page = topic_data.pages.get_sub_page(sub_page_id);

            if sub_page.is_none() {
                crate::operations::load_page_and_try_to_deliver_again(
                    app,
                    topic.clone(),
                    sub_page_id,
                );
                return;
            }

            compile_sub_page(
                sub_page_id,
                app.get_max_delivery_size(),
                topic_queue,
                &mut package_builder,
                sub_page.unwrap(),
            );

            if package_builder.data_size() > 0 {
                crate::operations::send_package::send_new_messages_to_deliver(
                    package_builder,
                    topic_data,
                );

                return;
            }
        }
    }
}

fn compile_sub_page(
    sub_page_id: SubPageId,
    max_delivery_size: usize,
    topic_queue: &mut TopicQueue,
    package_builder: &mut SubscriberPackageBuilder,
    sub_page: &SubPage,
) {
    while package_builder.data_size() < max_delivery_size {
        let message_id = topic_queue.queue.dequeue();

        if message_id.is_none() {
            break;
        }

        let message_id = message_id.unwrap();

        let msg_sub_page_id = SubPageId::from_message_id(message_id);

        if msg_sub_page_id.get_value() != sub_page_id.get_value() {
            break;
        };

        match sub_page.get_message(message_id) {
            my_service_bus_shared::sub_page::GetMessageResult::Message(msg) => {
                let attempt_no = topic_queue.delivery_attempts.get(msg.id);
                package_builder.add_message(msg, attempt_no);
            }
            my_service_bus_shared::sub_page::GetMessageResult::Missing => {}
            my_service_bus_shared::sub_page::GetMessageResult::Gced => return,
        }
    }
}

#[cfg(test)]
mod tests {

    use my_service_bus_abstractions::{publisher::MessageToPublish, subscriber::TopicQueueType};
    use my_service_bus_tcp_shared::TcpContract;

    use crate::{
        sessions::{SessionId, TestConnectionData},
        settings::SettingsModel,
    };

    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_publish_subsribe_case() {
        const TOPIC_NAME: &str = "test-topic";
        const QUEUE_NAME: &str = "test-queue";
        const SESSION_ID: SessionId = 13;
        const DELIVERY_SIZE: usize = 16;

        let settings = SettingsModel::create_test_settings(DELIVERY_SIZE);

        let app = Arc::new(AppContext::new(&settings).await);

        let session = app
            .sessions
            .add_test(TestConnectionData::new(SESSION_ID, "127.0.0.1"))
            .await;

        crate::operations::publisher::create_topic_if_not_exists(
            &app,
            Some(session.id),
            TOPIC_NAME,
        )
        .await
        .unwrap();

        crate::operations::subscriber::subscribe_to_queue(
            &app,
            TOPIC_NAME.to_string(),
            QUEUE_NAME.to_string(),
            TopicQueueType::Permanent,
            &session,
        )
        .await
        .unwrap();

        let msg1 = MessageToPublish {
            headers: None,
            content: vec![0u8, 1u8, 2u8],
        };

        let msg2 = MessageToPublish {
            headers: None,
            content: vec![3u8, 4u8, 5u8],
        };

        let messages = vec![msg1, msg2];

        crate::operations::publisher::publish(&app, TOPIC_NAME, messages, false, session.id)
            .await
            .unwrap();

        let test_connection = session.connection.unwrap_as_test();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let mut result_packets = test_connection.get_list_of_packets_and_clear_them().await;
        assert_eq!(result_packets.len(), 1);

        let packet = result_packets.remove(0);

        if let TcpContract::Raw(_) = packet {
        } else {
            panic!("Should not be here")
        }
    }

    /*
    #[tokio::test(flavor = "multi_thread")]
    async fn test_we_subscriber_and_deliver_persisted_messages() {
        const TOPIC_NAME: &str = "test-topic";
        const QUEUE_NAME: &str = "test-queue";
        const SESSION_ID: SessionId = 13;
        const DELIVERY_SIZE: usize = 16;

        let settings = SettingsModel::create_test_settings(DELIVERY_SIZE);

        let app = Arc::new(AppContext::new(&settings).await);

        let session = app
            .sessions
            .add_test(TestConnectionData::new(SESSION_ID, "127.0.0.1"))
            .await;

        app.topic_list.restore(TOPIC_NAME.to_string(), 3).await;

        //Simulate that we have persisted messages
        let msg1 = MessageProtobufModel {
            headers: vec![],
            data: vec![0u8, 1u8, 2u8],
            message_id: 1,
            created: DateTimeAsMicroseconds::now().unix_microseconds,
        };

        let msg2 = MessageProtobufModel {
            headers: vec![],
            data: vec![0u8, 1u8, 2u8],
            message_id: 2,
            created: DateTimeAsMicroseconds::now().unix_microseconds,
        };

        let messages_to_persist = vec![msg1, msg2];

        app.messages_pages_repo
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
                TOPIC_NAME.to_string(),
                QUEUE_NAME.to_string(),
                TopicQueueType::Permanent,
                queue_with_intervals,
            );
        }

        crate::operations::subscriber::subscribe_to_queue(
            &app,
            TOPIC_NAME.to_string(),
            QUEUE_NAME.to_string(),
            TopicQueueType::Permanent,
            &session,
        )
        .await
        .unwrap();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let version = session.get_message_to_delivery_protocol_version();

        let test_connection = session.connection.unwrap_as_test();

        let mut result_packets = test_connection.get_list_of_packets_and_clear_them().await;
        assert_eq!(result_packets.len(), 1);

        let packet = result_packets.remove(0);

        let packet =
            my_service_bus_tcp_shared::tcp_serializers::convert_from_raw(packet, &version).await;

        if let TcpContract::NewMessages {
            topic_id,
            queue_id,
            confirmation_id,
            messages,
        } = packet
        {
            assert_eq!(TOPIC_NAME, topic_id);
            assert_eq!(QUEUE_NAME, queue_id);
            println!("ConfirmationId: {}", confirmation_id);
            assert_eq!(2, messages.len());
        } else {
            panic!("Should not be here")
        }
    }
     */
}
