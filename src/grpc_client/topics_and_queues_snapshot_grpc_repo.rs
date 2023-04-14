use std::time::Duration;

use tokio_stream::StreamExt;
use tonic::transport::Channel;

use crate::topics::TopicSnapshot;

use crate::persistence_grpc::my_service_bus_queue_persistence_grpc_service_client::MyServiceBusQueuePersistenceGrpcServiceClient;
use crate::persistence_grpc::*;

use super::PersistenceError;

pub struct TopcsAndQueuesSnapshotGrpcRepo {
    channel: Channel,
    timeout: Duration,
}

impl TopcsAndQueuesSnapshotGrpcRepo {
    pub async fn new(grpc_address: String) -> Self {
        let channel = Channel::from_shared(grpc_address)
            .unwrap()
            .connect()
            .await
            .unwrap();
        Self {
            timeout: Duration::from_secs(5),
            channel,
        }
    }

    fn create_grpc_service(&self) -> MyServiceBusQueuePersistenceGrpcServiceClient<Channel> {
        MyServiceBusQueuePersistenceGrpcServiceClient::new(self.channel.clone())
    }

    pub async fn load(&self) -> Result<Vec<TopicSnapshot>, PersistenceError> {
        let mut grpc_client = self.create_grpc_service();

        let result = load_snapshot_with_timeout(&mut grpc_client, self.timeout).await;

        Ok(result)
    }

    pub async fn save(&self, snapshot: Vec<TopicSnapshot>) -> Result<(), PersistenceError> {
        let mut grpc_client = self.create_grpc_service();

        save_snapshot_with_timeout(&mut grpc_client, snapshot, self.timeout).await;

        Ok(())
    }
}

async fn save_snapshot_with_timeout(
    grpc: &mut MyServiceBusQueuePersistenceGrpcServiceClient<Channel>,
    snapshot: Vec<TopicSnapshot>,
    timeout: Duration,
) {
    let grpc_request: SaveQueueSnapshotGrpcRequest = snapshot.into();

    match tokio::time::timeout(timeout, grpc.save_snapshot(grpc_request)).await {
        Ok(result) => {
            result.unwrap();
            return;
        }
        Err(_) => panic!("save_snapshot timeout"),
    }
}

async fn load_snapshot_with_timeout(
    grpc: &mut MyServiceBusQueuePersistenceGrpcServiceClient<Channel>,
    timeout: Duration,
) -> Vec<TopicSnapshot> {
    let mut attempt_no = 0;

    loop {
        match tokio::time::timeout(timeout, grpc.get_snapshot(())).await {
            Ok(response) => {
                let response = response.unwrap();

                let mut response = response.into_inner();

                let mut result: Vec<TopicSnapshot> = Vec::new();

                while let Some(item) = response.next().await {
                    let grpc_model = item.unwrap();
                    result.push(grpc_model.into());
                }

                return result;
            }
            Err(_) => {
                if attempt_no >= 5 {
                    panic!("save_snapshot timeout");
                }

                attempt_no += 1;
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
    }
}
