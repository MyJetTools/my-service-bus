use std::sync::Arc;

use crate::{app::AppContext, http::middlewares::controllers::ControllersMiddleware};

pub fn build(app: Arc<AppContext>) -> ControllersMiddleware {
    let mut controllers = ControllersMiddleware::new();

    let topics_controller = Arc::new(super::topics::TopicsController::new(app.clone()));

    controllers.register_get_action("/Topics", topics_controller.clone());
    controllers.register_post_action("/Topics/Create", topics_controller);

    let connections_controller = super::connections::ConnectionsController::new(app.clone());

    controllers.register_delete_action(
        "/Connections/KickTcpConnection",
        Arc::new(connections_controller),
    );

    let greeting_controller = super::greeting::GreetingController::new(app.clone());
    controllers.register_post_action("/Greeting", Arc::new(greeting_controller));

    let greeting_ping_controller = super::greeting::PingController::new(app.clone());
    controllers.register_post_action("/Greeting/Ping", Arc::new(greeting_ping_controller));

    let status_controller = super::status::status_controller::StatusController::new(app.clone());
    controllers.register_get_action("/Status", Arc::new(status_controller));

    let queues_controller = Arc::new(super::queues::QueuesController::new(app.clone()));
    controllers.register_get_action("/Queues", queues_controller.clone());
    controllers.register_post_action("/Queues/SetMessageId", queues_controller.clone());
    controllers.register_delete_action("/Queues", queues_controller);

    let locks_controller = super::debug::LocksController::new(app.clone());
    controllers.register_get_action("/Locks", Arc::new(locks_controller));

    let debug_mode_controller = Arc::new(super::debug::DebugModeController::new(app.clone()));
    controllers.register_post_action("/Debug/Enable", debug_mode_controller.clone());
    controllers.register_delete_action("/Debug/Disable", debug_mode_controller.clone());

    let on_delivery_controller = Arc::new(super::debug::OnDeliveryController::new(app.clone()));
    controllers.register_get_action("/Debug/OnDelivery", on_delivery_controller);

    let logs_controller = Arc::new(super::logs::LogsController::new(app.clone()));
    controllers.register_get_action("/Logs", logs_controller);

    let logs_by_topic_controller = Arc::new(super::logs::LogsByTopicController::new(app.clone()));
    controllers.register_get_action("/Logs/Topic/{topicId}", logs_by_topic_controller);

    let logs_by_process_controller =
        Arc::new(super::logs::LogsByProcessController::new(app.clone()));
    controllers.register_get_action("/Logs/Process/{processId}", logs_by_process_controller);

    controllers
}
