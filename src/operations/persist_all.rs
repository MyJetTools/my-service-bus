use std::sync::Arc;

use my_logger::LogEventCtx;

use crate::{
    app::AppContext,
    persistence_grpc::{
        QueueIndexRangeGrpcModel, QueueSnapshotGrpcModel, TopicAndQueuesSnapshotGrpcModel,
    },
};

pub async fn persist_all(app: &Arc<AppContext>) {
    let namespaces = app.namespaces.get_all();

    // Every namespace goes into one stream: persistence keeps a single snapshot
    // where each record names its namespace, and that is also how the node learns
    // which namespaces exist when it restores.
    let mut topics_snapshots = Vec::new();

    for namespace in namespaces.iter() {
        let grpc_namespace = namespace.as_grpc_namespace();

        for topic in namespace.topic_list.get_all().iter() {
            topics_snapshots.push(topic.get_topic_info(|topic_data| {
                TopicAndQueuesSnapshotGrpcModel {
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
                    deleted: topic_data.deleted,
                    namespace: grpc_namespace.clone(),
                }
            }));
        }
    }

    let result = app
        .persistence_client
        .save_topic_and_queues(topics_snapshots)
        .await;

    if let Err(err) = result {
        my_logger::LOGGER.write_error(
            "persist_all",
            format!("Failed to save topics and queues snapshot: {:?}", err),
            LogEventCtx::new(),
        );
    }

    for namespace in namespaces.iter() {
        for topic in namespace.topic_list.get_all().iter() {
            crate::operations::persist_topic_messages(app, topic).await;
        }
    }
}
