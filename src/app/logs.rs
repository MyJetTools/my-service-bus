use std::{collections::HashMap, sync::Arc};

use rust_extensions::{date_time::DateTimeAsMicroseconds, Logger};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    RwLock,
};

#[derive(Debug, Clone, Copy)]
pub enum SystemProcess {
    System = 0,
    TcpSocket = 1,
    TopicOperation = 2,
    QueueOperation = 3,
    Init = 4,
    Timer = 5,
    Persistence = 6,
    DeliveryOperation = 7,
}

impl SystemProcess {
    pub fn iterate() -> Vec<Self> {
        let mut result = Vec::new();

        result.push(SystemProcess::System);
        result.push(SystemProcess::TcpSocket);
        result.push(SystemProcess::TopicOperation);
        result.push(SystemProcess::QueueOperation);
        result.push(SystemProcess::Init);
        result.push(SystemProcess::Timer);
        result.push(SystemProcess::Persistence);
        result.push(SystemProcess::DeliveryOperation);

        return result;
    }
    pub fn parse(value: &str) -> Option<Self> {
        if value == "system" {
            return Some(SystemProcess::System);
        }

        if value == "tcpsocket" {
            return Some(SystemProcess::TcpSocket);
        }

        if value == "topicpperation" {
            return Some(SystemProcess::TopicOperation);
        }

        if value == "queueoperation" {
            return Some(SystemProcess::QueueOperation);
        }

        if value == "init" {
            return Some(SystemProcess::Init);
        }

        if value == "timer" {
            return Some(SystemProcess::Timer);
        }

        if value == "persistence" {
            return Some(SystemProcess::Persistence);
        }

        if value == "deliveryoperation" {
            return Some(SystemProcess::DeliveryOperation);
        }

        return None;
    }

    pub fn as_u8(&self) -> u8 {
        let result = *self as u8;
        return result;
    }
}

#[derive(Debug, Clone)]
pub enum LogLevel {
    Info,
    Error,
    FatalError,
}
#[derive(Debug, Clone)]
pub struct LogItem {
    pub date: DateTimeAsMicroseconds,

    pub topic: Option<String>,

    pub level: LogLevel,

    pub process: SystemProcess,

    pub process_name: String,

    pub message: String,

    pub err_ctx: Option<HashMap<String, String>>,
}

struct LogsData {
    items: Vec<Arc<LogItem>>,
    items_by_topic: HashMap<String, Vec<Arc<LogItem>>>,
    items_by_process: HashMap<u8, Vec<Arc<LogItem>>>,
}

impl LogsData {
    async fn add(&mut self, item: LogItem) {
        let item = Arc::new(item);

        let process_id = item.as_ref().process.as_u8();

        add_topic_data(&mut self.items_by_process, &process_id, item.clone());

        if let Some(topic_name) = &item.topic {
            add_topic_data(&mut self.items_by_topic, topic_name, item.clone());
        }

        let items = &mut self.items;
        items.push(item);
        gc_logs(items);
    }
}

pub struct Logs {
    data: Arc<RwLock<LogsData>>,
    sender: UnboundedSender<LogItem>,
}

impl Logs {
    pub fn new() -> Self {
        let logs_data = LogsData {
            items: Vec::new(),
            items_by_topic: HashMap::new(),
            items_by_process: HashMap::new(),
        };
        let logs_data = Arc::new(RwLock::new(logs_data));

        let (sender, recv) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(log_writer_thread(recv, logs_data.clone()));

        Self {
            data: logs_data,
            sender,
        }
    }

    pub fn add_info(
        &self,
        topic: Option<String>,
        process: SystemProcess,
        process_name: String,
        message: String,
        context: Option<HashMap<String, String>>,
    ) {
        let item = LogItem {
            date: DateTimeAsMicroseconds::now(),
            level: LogLevel::Info,
            topic,
            process_name,
            process,
            message: message,
            err_ctx: context,
        };

        self.add_item(item);
    }

    fn add_item(&self, item: LogItem) {
        let result = self.sender.send(item);

        if let Err(err) = result {
            println!("Can not write log item. Reason: {:?}", err);
        }
    }

    pub fn add_error(
        &self,
        topic: Option<String>,
        process: SystemProcess,
        process_name: String,
        message: String,
        err_ctx: Option<HashMap<String, String>>,
    ) {
        let item = LogItem {
            date: DateTimeAsMicroseconds::now(),
            level: LogLevel::Error,
            topic,
            process_name,
            process,
            message: message,
            err_ctx,
        };

        println!(
                "{dt} {level:?} {proces:?}\n Topic:{topic:?}\n Process:{processname}\n Message:{message}\n Ctx:{err_ctx:?}",
                topic= item.topic,
                dt = item.date.to_rfc3339(),
                level = item.level,
                proces = item.process,
                processname = item.process_name,
                message = item.message,
                err_ctx = item.err_ctx
            );
        println!("-------------");
        self.add_item(item);
    }

    pub fn add_fatal_error(
        &self,
        process: SystemProcess,
        process_name: String,
        message: String,
        context: Option<HashMap<String, String>>,
    ) {
        let item = LogItem {
            date: DateTimeAsMicroseconds::now(),
            level: LogLevel::FatalError,
            topic: None,
            process_name,
            process,
            message,
            err_ctx: context,
        };

        println!(
            "{dt} {level:?} {proces:?}\n Process:{processname}\n Message:{message}",
            dt = item.date.to_rfc3339(),
            level = item.level,
            proces = item.process,
            processname = item.process_name,
            message = item.message
        );
        println!("-------------");

        self.add_item(item);
    }

    pub async fn get(&self) -> Vec<Arc<LogItem>> {
        let read_access = self.data.read().await;
        read_access.items.to_vec()
    }

    pub async fn get_by_topic(&self, topic: &str) -> Option<Vec<Arc<LogItem>>> {
        let read_access = self.data.read().await;
        let result = read_access.items_by_topic.get(topic)?;
        return Some(result.to_vec());
    }

    pub async fn get_by_process(&self, process: SystemProcess) -> Option<Vec<Arc<LogItem>>> {
        let read_access = self.data.read().await;
        let result = read_access.items_by_process.get(&process.as_u8())?;
        return Some(result.to_vec());
    }
}

fn add_topic_data<T>(
    items_by_topic: &mut HashMap<T, Vec<Arc<LogItem>>>,
    category: &T,
    item: Arc<LogItem>,
) where
    T: Eq + std::hash::Hash + Clone + Sized,
{
    if !items_by_topic.contains_key(category) {
        items_by_topic.insert(category.clone(), Vec::new());
    }

    let items = items_by_topic.get_mut(category);

    if let Some(items) = items {
        items.push(item);
        gc_logs(items);
    }
}

fn gc_logs(items: &mut Vec<Arc<LogItem>>) {
    while items.len() > 100 {
        items.remove(0);
    }
}

async fn log_writer_thread(mut recv: UnboundedReceiver<LogItem>, logs_data: Arc<RwLock<LogsData>>) {
    while let Some(next_item) = recv.recv().await {
        let mut write_access = logs_data.as_ref().write().await;
        write_access.add(next_item).await;
    }
}

impl Logger for Logs {
    fn write_info(&self, process: String, message: String, ctx: Option<HashMap<String, String>>) {
        self.add_info(None, SystemProcess::System, process, message, ctx);
    }

    fn write_warning(
        &self,
        process: String,
        message: String,
        ctx: Option<HashMap<String, String>>,
    ) {
        self.add_info(
            None,
            SystemProcess::System,
            process,
            format!("Warning: {message}"),
            ctx,
        );
    }

    fn write_error(&self, process: String, message: String, ctx: Option<HashMap<String, String>>) {
        self.add_error(None, SystemProcess::System, process, message, ctx);
    }

    fn write_fatal_error(
        &self,
        process: String,
        message: String,
        ctx: Option<HashMap<String, String>>,
    ) {
        self.add_fatal_error(SystemProcess::System, process, message, ctx);
    }
}
