use std::{sync::Arc, time::Duration};

use my_logger::LogEventCtx;
use my_service_bus::abstractions::queue_with_intervals::QueueWithIntervals;
use rust_extensions::StopWatch;

use crate::topics::{Topic, TopicSnapshot};

use crate::app::AppContext;

pub async fn init(app: Arc<AppContext>) {
    let mut sw = StopWatch::new();
    sw.start();

    let topics_and_queues = restore_topics_and_queues(app.as_ref()).await;

    println!("Loaded topics {}", topics_and_queues.len());

    for topic_and_queues in topics_and_queues {
        let topic = app
            .topic_list
            .restore(
                topic_and_queues.topic_id.as_str(),
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

    my_logger::LOGGER.write_info(
        "Initialization",
        format!("Initialization is done in {:?}", sw.duration()),
        LogEventCtx::new(),
    );

    println!("Application is initialized in {:?}", sw.duration());
}

async fn restore_topic_pages(app: Arc<AppContext>, topic: Arc<Topic>) {
    let sub_page_id = topic.get_current_sub_page().await;

    let sub_page = crate::operations::page_loader::load_page_to_cache(
        &topic,
        app.messages_pages_repo.clone(),
        sub_page_id,
    )
    .await;

    if let Some(sub_page) = sub_page {
        let mut topic_data = topic.get_access().await;
        topic_data.pages.restore_sub_page(sub_page);
    }
}

async fn restore_topics_and_queues(app: &AppContext) -> Vec<TopicSnapshot> {
    let mut attempt = 0;
    loop {
        attempt += 1;

        let topics_and_queues = app.topics_and_queues_repo.load().await;

        my_logger::LOGGER.write_info(
            "restore_topics_and_queues",
            format!("Restoring topics and queues"),
            LogEventCtx::new().add("attemptNo", attempt.to_string()),
        );

        if let Ok(result) = topics_and_queues {
            return result;
        }

        let err = topics_and_queues.err().unwrap();

        my_logger::LOGGER.write_error(
            "restore_topics_and_queues",
            format!("Can not restore topics and queues. Err: {:?}", err),
            LogEventCtx::new().add("attemptNo", attempt.to_string()),
        );

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
