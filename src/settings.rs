use std::time::Duration;

use my_grpc_extensions::*;
use rust_extensions::duration_utils::DurationExtensions;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};

#[cfg(test)]
const TEST_GRPC_URL: &str = "test";

#[derive(Serialize, Deserialize, Debug)]
pub struct SettingsModelYaml {
    pub persistence_grpc_url: String,

    pub queue_gc_timeout: String,

    pub debug_mode: bool,

    pub max_delivery_size: usize,

    pub delivery_timeout: Option<String>,

    pub auto_create_topic_on_publish: Option<bool>,

    pub auto_create_topic_on_subscribe: Option<bool>,

    pub grpc_timeout_secs: u64,

    pub persist_timer_interval: String,

    pub listen_unix_socket: Option<String>,
}

pub struct SettingsModel {
    pub persistence_grpc_url: String,
    pub queue_gc_timeout: Duration,

    pub max_delivery_size: usize,

    pub delivery_timeout: Option<Duration>,

    pub auto_create_topic_on_publish: bool,
    pub auto_create_topic_on_subscribe: bool,
    pub persist_timer_interval: Duration,

    pub listen_unix_socket: Option<String>,
}

#[async_trait::async_trait]
impl GrpcClientSettings for SettingsModel {
    async fn get_grpc_url(&self, _name: &'static str) -> GrpcUrl {
        self.persistence_grpc_url.clone().into()
    }
}

impl SettingsModel {
    pub async fn read() -> Self {
        let filename = get_settings_filename();

        println!("Reading settings file {}", filename);

        let file = File::open(&filename).await;

        if let Err(err) = file {
            panic!(
                "Can not open settings file: {}. The reason is: {:?}",
                filename, err
            );
        }

        let mut file = file.unwrap();

        let mut file_content: Vec<u8> = Vec::new();

        loop {
            let res = file.read_buf(&mut file_content).await.unwrap();

            if res == 0 {
                break;
            }
        }

        let result: SettingsModelYaml = serde_yaml::from_slice(&file_content).unwrap();

        result.into()
    }

    #[cfg(test)]
    pub fn create_test_settings(max_delivery_size: usize) -> Self {
        Self {
            persistence_grpc_url: TEST_GRPC_URL.to_string(),
            queue_gc_timeout: Duration::from_secs(1),
            max_delivery_size,
            delivery_timeout: None,
            auto_create_topic_on_publish: true,
            auto_create_topic_on_subscribe: true,
            persist_timer_interval: Duration::from_secs(1),
            listen_unix_socket: None,
        }
    }
}

#[cfg(target_os = "windows")]
fn get_settings_filename() -> String {
    let home_path = env!("HOME");
    let filename = format!("{}\\{}", home_path, ".myservicebus");
    filename
}

#[cfg(not(target_os = "windows"))]
fn get_settings_filename() -> String {
    let home_path = env!("HOME");
    let filename = format!("{}/{}", home_path, ".myservicebus");
    filename
}

impl Into<SettingsModel> for SettingsModelYaml {
    fn into(self) -> SettingsModel {
        let queue_gc_timeout =
            rust_extensions::duration_utils::parse_duration(self.queue_gc_timeout.as_str())
                .unwrap();

        let delivery_timeout = if let Some(src) = self.delivery_timeout {
            println!("Delivery timeout is set {}", src);

            let timeout_duration = rust_extensions::duration_utils::parse_duration(src.as_str());

            if let Err(err) = timeout_duration {
                panic!(
                    "Can not parse Delivery Timeout value '{}'. Reason: {:?}",
                    src, err
                );
            }
            Some(timeout_duration.unwrap())
        } else {
            println!(
                "Delivery timeout is disabled. To enable please specify DeliveryTimeout: hh:mm:ss"
            );
            None
        };

        let auto_create_topic_on_publish = if let Some(auto_create_topic) =
            self.auto_create_topic_on_publish
        {
            if auto_create_topic {
                println!("Auto create topic on publish is enabled");
            } else {
                println!("Auto create topic on publish is disabled");
            }

            auto_create_topic
        } else {
            println!("Auto create topic on publish is disabled. To enable please add parameter AutoCreateTopicOnPublish: true");
            false
        };

        let auto_create_topic_on_subscribe = if let Some(auto_create_topic_on_subscribe) =
            self.auto_create_topic_on_subscribe
        {
            if auto_create_topic_on_subscribe {
                println!("Auto create topic on subscribe is enabled");
            } else {
                println!("Auto create topic on subscribe is disabled");
            }

            auto_create_topic_on_subscribe
        } else {
            println!("Auto create topic on subscribe is disabled. To enable please add parameter AutoCreateTopicOnSubscribe: true");
            false
        };

        SettingsModel {
            persistence_grpc_url: self.persistence_grpc_url,
            queue_gc_timeout,
            max_delivery_size: self.max_delivery_size,
            delivery_timeout,
            auto_create_topic_on_publish,
            auto_create_topic_on_subscribe,
            persist_timer_interval: Duration::from_str(&self.persist_timer_interval).unwrap(),
            listen_unix_socket: self.listen_unix_socket,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use rust_extensions::duration_utils::DurationExtensions;

    #[test]
    fn test() {
        let diration = Duration::from_str("100ms").unwrap();

        println!("{:?}", diration);
    }
}
