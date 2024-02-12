use std::time::Duration;

use rust_extensions::{date_time::DateTimeAsMicroseconds, TaskCompletion, TaskCompletionAwaiter};

use super::{HttpDeliveryPackage, HttpSendQueue};

pub struct SendQueueInner {
    pub queue: HttpSendQueue,
    pub task_completion: Option<(
        TaskCompletion<Option<HttpDeliveryPackage>, String>,
        DateTimeAsMicroseconds,
    )>,
}

impl SendQueueInner {
    pub fn new() -> Self {
        Self {
            queue: HttpSendQueue::new(),
            task_completion: None,
        }
    }

    pub fn deliver_message(&mut self) {
        if self.task_completion.is_none() {
            return;
        }

        if let Some(package) = self.queue.get_next_package() {
            let (mut task_completion, _) = self.task_completion.take().unwrap();
            task_completion.set_ok(Some(package));
        }
    }

    pub fn engage_awaiter(&mut self) -> TaskCompletionAwaiter<Option<HttpDeliveryPackage>, String> {
        if self.task_completion.is_some() {
            panic!("Task completion is already engaged");
        }

        let mut task_completion = TaskCompletion::new();

        let awaiter = task_completion.get_awaiter();
        self.task_completion = Some((task_completion, DateTimeAsMicroseconds::now()));
        awaiter
    }

    pub fn ping_awaiter(&mut self) {
        let dispose_it = if let Some((_, created)) = &self.task_completion.take() {
            let now = DateTimeAsMicroseconds::now();

            now.duration_since(*created).as_positive_or_zero() > Duration::from_secs(5)
        } else {
            false
        };

        if dispose_it {
            if let Some((mut task_completion, _)) = self.task_completion.take() {
                task_completion.set_ok(None)
            }
        }
    }
}
