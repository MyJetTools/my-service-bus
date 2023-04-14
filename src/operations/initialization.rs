use std::{sync::Arc, time::Duration};

use my_service_bus_abstractions::queue_with_intervals::QueueWithIntervals;
use rust_extensions::StopWatch;

use crate::topics::{Topic, TopicSnapshot};

use crate::app::AppContext;

pub async fn init(app: Arc<AppContext>) {
    let mut sw = StopWatch::new();
    sw.start();

    let topics_and_queues = restore_topics_and_queues(app.as_ref()).await;

    println!("Loaded topics {}", topics_and_queues.len());

    let topics_count = topics_and_queues.len();

    for topic_and_queues in topics_and_queues {
        let topic = app
            .topic_list
            .restore(
                topic_and_queues.topic_id,
                topic_and_queues.message_id.into(),
            )
            .await;

        for queue in topic_and_queues.queues {
            let queue_with_intervals = QueueWithIntervals::restore(queue.ranges);

            let mut topic_data = topic.get_access().await;
            topic_data.queues.restore(
                topic.topic_id.to_string(),
                queue.queue_id.to_string(),
                queue.queue_type,
                queue_with_intervals,
            );
        }
    }

    for topic in app.topic_list.get_all().await {
        restore_topic_pages(app.clone(), topic.clone()).await;
    }

    app.states.set_initialized();
    sw.pause();

    app.logs.add_info(
        None,
        crate::app::logs::SystemProcess::Init,
        format!("Initialization is done in {:?}", sw.duration()),
        format!(
            "Application is initialized. Topics amount is: {}",
            topics_count
        ),
        None,
    );

    println!("Application is initialized in {:?}", sw.duration());
}

async fn restore_topic_pages(app: Arc<AppContext>, topic: Arc<Topic>) {
    let (page_id, sub_page_id) = topic.get_current_page().await;

    let sub_page = crate::operations::page_loader::load_page_to_cache(
        &topic,
        app.messages_pages_repo.clone(),
        Some(app.logs.as_ref()),
        page_id,
        sub_page_id,
    )
    .await;

    let mut topic_data = topic.get_access().await;
    topic_data.pages.restore_sub_page(sub_page);
}

async fn restore_topics_and_queues(app: &AppContext) -> Vec<TopicSnapshot> {
    let mut attempt = 0;
    loop {
        attempt += 1;

        let topics_and_queues = app.topics_and_queues_repo.load().await;

        app.logs.add_info(
            None,
            crate::app::logs::SystemProcess::Init,
            "restore_topics_and_queues".to_string(),
            format!("Restoring topics and queues. Attempt {}", attempt),
            None,
        );

        if let Ok(result) = topics_and_queues {
            return result;
        }

        let err = topics_and_queues.err().unwrap();

        app.logs.add_error(
            None,
            crate::app::logs::SystemProcess::Init,
            "restore_topics_and_queues".to_string(),
            format!("Can not restore topics and queues. Err: {:?}", err),
            None,
        );

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
