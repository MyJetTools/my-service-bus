use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Serialize, Deserialize, Debug)]
pub struct SettingsModelJson {
    #[serde(rename = "GrpcUrl")]
    pub persistence_grpc_url: String,

    #[serde(rename = "EventuallyPersistenceDelay")]
    pub eventually_persistence_delay: String,

    #[serde(rename = "QueueGcTimeout")]
    pub queue_gc_timeout: String,

    #[serde(rename = "DebugMode")]
    pub debug_mode: bool,

    #[serde(rename = "MaxDeliverySize")]
    pub max_delivery_size: usize,

    #[serde(rename = "DeliveryTimeout")]
    pub delivery_timeout: Option<String>,

    #[serde(rename = "AutoCreateTopic")]
    pub auto_create_topic: Option<bool>,
}

pub struct SettingsModel {
    pub persistence_grpc_url: String,
    pub eventually_persistence_delay: Duration,
    pub queue_gc_timeout: Duration,
    pub debug_mode: bool,

    pub max_delivery_size: usize,

    pub delivery_timeout: Option<Duration>,

    pub auto_create_topic: bool,
}

pub async fn read() -> SettingsModel {
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

    let result: SettingsModelJson = serde_yaml::from_slice(&file_content).unwrap();

    result.into()
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

impl Into<SettingsModel> for SettingsModelJson {
    fn into(self) -> SettingsModel {
        let queue_gc_timeout =
            rust_extensions::duration_utils::parse_duration(self.queue_gc_timeout.as_str())
                .unwrap();

        let eventually_persistence_delay = rust_extensions::duration_utils::parse_duration(
            self.eventually_persistence_delay.as_str(),
        )
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

        let auto_create_topic = if let Some(auto_create_topic) = self.auto_create_topic {
            if auto_create_topic {
                println!("Auto create topic on publish is enabled");
            } else {
                println!("Auto create topic on publish is disabled");
            }

            auto_create_topic
        } else {
            println!("Auto create topic on publish is disabled. To enable please add parameter AutoCreateTopic: true");
            false
        };

        SettingsModel {
            persistence_grpc_url: self.persistence_grpc_url,
            debug_mode: self.debug_mode,
            queue_gc_timeout,
            eventually_persistence_delay,
            max_delivery_size: self.max_delivery_size,
            delivery_timeout,
            auto_create_topic,
        }
    }
}
