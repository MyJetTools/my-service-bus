use std::net::IpAddr;
use std::sync::OnceLock;
use std::time::Duration;

use my_grpc_extensions::*;
use serde::{Deserialize, Serialize};
use tokio::{fs::File, io::AsyncReadExt};

#[cfg(test)]
const TEST_GRPC_URL: &str = "test";

/// Where the TCP and HTTP listeners bind when `listen_ip` is not set: every interface, as the
/// node always has.
const DEFAULT_LISTEN_IP: IpAddr = IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED);

#[derive(Serialize, Deserialize, Debug)]
pub struct SettingsModelYaml {
    pub persistence_grpc_url: String,

    pub queue_gc_timeout: String,

    pub debug_mode: bool,

    pub max_delivery_size: usize,

    pub delivery_timeout: Option<String>,

    pub auto_create_topic_on_publish: Option<bool>,

    pub auto_create_topic_on_subscribe: Option<bool>,

    pub listen_unix_socket: Option<String>,

    /// The address the TCP port (6421) and the HTTP port (6123) bind to. Absent: `0.0.0.0`.
    /// Neither port asks who is calling, so `127.0.0.1` is how a node is kept to its own machine.
    pub listen_ip: Option<String>,
}

pub struct SettingsModel {
    pub persistence_grpc_url: String,
    pub queue_gc_timeout: Duration,

    pub max_delivery_size: usize,

    pub delivery_timeout: Option<Duration>,

    pub auto_create_topic_on_publish: bool,
    pub auto_create_topic_on_subscribe: bool,

    pub listen_unix_socket: Option<String>,

    pub listen_ip: IpAddr,
}

/// The settings file named on the command line (`--settings <file>`, Windows), read instead of
/// `$HOME/.myservicebus`.
static EXPLICIT_SETTINGS_FILE: OnceLock<String> = OnceLock::new();

/// Names the settings file for the rest of the process; called once, before the settings are read.
#[cfg(windows)]
pub fn set_settings_file(file: String) -> Result<(), String> {
    EXPLICIT_SETTINGS_FILE
        .set(file)
        .map_err(|_| "The settings file is named once".to_string())
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
            listen_unix_socket: None,
            listen_ip: DEFAULT_LISTEN_IP,
        }
    }
}

/// `--settings <file>` when it was given, otherwise `.myservicebus` in the home directory of
/// whoever runs the node - read when the node starts. It used to be `env!("HOME")`, which is the
/// home directory of the machine the binary was BUILT on, baked in at compile time.
fn get_settings_filename() -> String {
    if let Some(file) = EXPLICIT_SETTINGS_FILE.get() {
        return file.clone();
    }

    let home_path = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| {
            panic!("Neither HOME nor USERPROFILE is set: the settings file .myservicebus is looked for there")
        });

    std::path::Path::new(home_path.as_str())
        .join(".myservicebus")
        .to_string_lossy()
        .to_string()
}

fn parse_listen_ip(value: Option<String>) -> IpAddr {
    match value {
        None => DEFAULT_LISTEN_IP,
        Some(value) => value.trim().parse().unwrap_or_else(|err| {
            panic!("listen_ip '{value}' is not an IP address: {err}");
        }),
    }
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
            listen_unix_socket: self.listen_unix_socket,
            listen_ip: parse_listen_ip(self.listen_ip),
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

    #[test]
    fn listen_ip_is_every_interface_unless_set() {
        assert_eq!(
            super::parse_listen_ip(None),
            std::net::IpAddr::from([0, 0, 0, 0])
        );
        assert_eq!(
            super::parse_listen_ip(Some(" 127.0.0.1 ".to_string())),
            std::net::IpAddr::from([127, 0, 0, 1])
        );
    }

    #[test]
    #[should_panic(expected = "is not an IP address")]
    fn a_listen_ip_which_is_no_address_stops_the_start() {
        super::parse_listen_ip(Some("localhost".to_string()));
    }
}
