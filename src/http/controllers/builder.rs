use std::sync::Arc;

use my_http_server_controllers::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: Arc<AppContext>) -> ControllersMiddleware {
    let mut controllers = ControllersMiddleware::new(None, None);

    controllers.register_get_action(Arc::new(super::topics::GetTopicsAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::topics::CreateTopicsAction::new(
        app.clone(),
    )));

    controllers.register_delete_action(Arc::new(super::sessions::DeleteSessionAction::new(
        app.clone(),
    )));

    controllers.register_post_action(Arc::new(super::greeting::GreetingAction::new(app.clone())));
    //controllers.register_http_objects(greeting_controller);

    controllers.register_post_action(Arc::new(super::greeting::PingAction::new(app.clone())));

    controllers.register_get_action(Arc::new(
        super::status::status_controller::GetStatusAction::new(app.clone()),
    ));

    controllers.register_get_action(Arc::new(super::queues::GetQueuesAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::queues::SetMessageIdAction::new(
        app.clone(),
    )));

    controllers
        .register_delete_action(Arc::new(super::queues::DeleteQueueAction::new(app.clone())));

    controllers.register_post_action(Arc::new(super::debug::EnableDebugModeAction::new(
        app.clone(),
    )));
    controllers.register_delete_action(Arc::new(super::debug::DisableDebugModeAction::new(
        app.clone(),
    )));

    let on_delivery_controller = Arc::new(super::debug::GetOnDeliveryAction::new(app.clone()));
    controllers.register_get_action(on_delivery_controller);

    let logs_controller = Arc::new(super::logs::GetLogsAction::new(app.clone()));
    controllers.register_get_action(logs_controller);

    let logs_by_topic_controller = Arc::new(super::logs::GetLogsByTopicAction::new(app.clone()));
    controllers.register_get_action(logs_by_topic_controller);

    let logs_by_process_controller =
        Arc::new(super::logs::GetLogsByProcessAction::new(app.clone()));
    controllers.register_get_action(logs_by_process_controller);

    let publisher_controller = super::publisher::PublishAction::new(app.clone());
    controllers.register_post_action(Arc::new(publisher_controller));

    controllers.register_get_action(Arc::new(super::home_controller::IndexAction::new(
        app.clone(),
    )));

    controllers.register_get_action(Arc::new(super::prometheus_controller::MetricsAction::new(
        app,
    )));

    controllers
}
