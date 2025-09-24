use std::sync::Arc;

use my_logger::LogEventCtx;

use crate::{
    app::AppContext,
    persistence_grpc::{
        QueueIndexRangeGrpcModel, QueueSnapshotGrpcModel, TopicAndQueuesSnapshotGrpcModel,
    },
    topics::ReusableTopicsList,
};

pub async fn persist_topics_and_queues(
    app: &Arc<AppContext>,
    reusable_topics: &mut ReusableTopicsList,
) {
    if let Some(get_persistence_version) = app.persistence_client.get_persistence_version().await {
        app.persistence_version
            .update(get_persistence_version.as_str())
            .await;
    }

    app.topic_list.fill_topics(reusable_topics).await;

    let mut topics_snapshots = Vec::with_capacity(reusable_topics.len());

    for topic in reusable_topics.iter() {
        topics_snapshots.push(
            topic
                .get_topic_info(|topic_data| TopicAndQueuesSnapshotGrpcModel {
                    topic_id: topic_data.topic_id.to_string(),
                    message_id: topic_data.message_id.get_value(),
                    queue_snapshots: topic_data
                        .queues
                        .get_snapshot(|itm| QueueSnapshotGrpcModel {
                            queue_id: itm.queue_id.to_string(),
                            ranges: itm
                                .queue
                                .get_intervals()
                                .iter()
                                .map(|itm| QueueIndexRangeGrpcModel {
                                    from_id: itm.from_id,
                                    to_id: itm.to_id,
                                })
                                .collect(),
                            queue_type: itm.queue_type.into_u8() as i32,
                        }),
                    persist: Some(topic_data.persist),
                })
                .await,
        );
    }

    let result = app
        .persistence_client
        .save_topic_and_queues(topics_snapshots)
        .await;

    if let Err(err) = result {
        my_logger::LOGGER.write_error(
            "persist_topics_and_queues",
            format!("Failed to save topics and queues snapshot: {:?}", err),
            LogEventCtx::new(),
        );
    }

    for topic in reusable_topics.iter() {
        crate::operations::persist_topic_messages(&app, topic).await;
    }
}
