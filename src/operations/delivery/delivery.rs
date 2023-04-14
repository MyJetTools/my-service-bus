use my_service_bus_abstractions::AsMessageId;
use my_service_bus_shared::{
    page_id::PageId,
    sub_page::{GetMessageResult, SubPageId},
};

use std::sync::Arc;

use crate::{
    app::AppContext,
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
            subscriber.id,
            subscriber.session.clone(),
        );

        return Some(result);
    }

    None
}

pub async fn continue_delivering(
    app: &Arc<AppContext>,
    topic: &Arc<Topic>,
    package_builder: SubscriberPackageBuilder,
) {
    let mut topic_data = topic.get_access().await;
    let queue = topic_data.queues.get_mut(package_builder.queue_id.as_str());

    if queue.is_none() {
        return;
    }

    let queue = queue.unwrap();

    let subscriber = queue
        .subscribers
        .get_by_id_mut(package_builder.subscriber_id);

    if subscriber.is_none() {
        return;
    }

    compile_and_deliver(app, package_builder, topic, &mut topic_data);
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
        while package_builder.get_data_size() < app.get_max_delivery_size() {
            let message_id = topic_queue.queue.peek();

            if message_id.is_none() {
                break;
            }

            let message_id = message_id.unwrap();

            let page_id = PageId::from_message_id(message_id.as_message_id());
            let sub_page_id = SubPageId::from_message_id(message_id.as_message_id());

            let page = topic_data.pages.get_page(page_id);

            if page.is_none() {
                start_loading(
                    app,
                    topic,
                    topic_data,
                    page_id,
                    sub_page_id,
                    package_builder,
                );

                return;
            }

            let page = page.unwrap();

            let sub_page = page.get_sub_page(&sub_page_id);

            if sub_page.is_none() {
                start_loading(
                    app,
                    topic,
                    topic_data,
                    page_id,
                    sub_page_id,
                    package_builder,
                );

                return;
            }

            let sub_page = sub_page.unwrap();

            topic_queue.queue.dequeue();

            match sub_page.sub_page.get_message(message_id.as_message_id()) {
                GetMessageResult::Message(message_content) => {
                    let attempt_no = topic_queue.delivery_attempts.get(message_content.id);
                    package_builder.add_message(message_content, attempt_no);
                }
                GetMessageResult::Missing => {}
                GetMessageResult::GarbageCollected => {
                    start_loading(
                        app,
                        topic,
                        topic_data,
                        page_id,
                        sub_page_id,
                        package_builder,
                    );
                    return;
                }
            }
        }
    }

    crate::operations::send_package::send_new_messages_to_deliver(package_builder, topic_data);
}

fn start_loading(
    app: &Arc<AppContext>,
    topic: &Arc<Topic>,
    topic_data: &mut TopicData,
    page_id: PageId,
    sub_page_id: SubPageId,
    package_builder: SubscriberPackageBuilder,
) {
    if package_builder.get_data_size() > 0 {
        crate::operations::send_package::send_new_messages_to_deliver(package_builder, topic_data);

        crate::operations::load_page_and_try_to_deliver_again(
            app,
            topic.clone(),
            page_id,
            sub_page_id,
            None,
        );
    } else {
        crate::operations::load_page_and_try_to_deliver_again(
            app,
            topic.clone(),
            page_id,
            sub_page_id,
            Some(package_builder),
        );
    }
}

#[cfg(test)]
mod tests {

    use my_service_bus_abstractions::{
        publisher::MessageToPublish, queue_with_intervals::QueueWithIntervals,
        subscriber::TopicQueueType,
    };
    use my_service_bus_shared::protobuf_models::MessageProtobufModel;
    use my_service_bus_tcp_shared::TcpContract;
    use rust_extensions::date_time::DateTimeAsMicroseconds;

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

        app.topic_list
            .restore(TOPIC_NAME.to_string(), 3.into())
            .await;

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
}
