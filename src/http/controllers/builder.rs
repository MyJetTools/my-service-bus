use std::sync::Arc;

use my_http_server_controllers::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: &Arc<AppContext>) -> ControllersMiddleware {
    let mut controllers = ControllersMiddleware::new(None, None);

    controllers.register_post_action(Arc::new(super::topics_controller::CreateTopicAction::new(
        app.clone(),
    )));
    controllers.register_get_action(Arc::new(super::topics_controller::GetTopicsAction::new(
        app.clone(),
    )));

    controllers.register_delete_action(Arc::new(super::topics_controller::DeleteTopicAction::new(
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

    controllers.register_post_action(Arc::new(
        super::debug_controller::EnableDebugModeAction::new(app.clone()),
    ));
    controllers.register_delete_action(Arc::new(
        super::debug_controller::DisableDebugModeAction::new(app.clone()),
    ));

    let on_delivery_controller =
        Arc::new(super::debug_controller::OnDeliveryAction::new(app.clone()));
    controllers.register_get_action(on_delivery_controller);

    let logs_controller = Arc::new(super::logs::LogsAction::new(app.clone()));
    controllers.register_get_action(logs_controller);

    controllers.register_get_action(Arc::new(super::logs::GetLogsByTopicAction::new(
        app.clone(),
    )));

    controllers.register_get_action(Arc::new(super::logs::GetLogsByProcessAction::new(
        app.clone(),
    )));

    controllers.register_get_action(Arc::new(super::logs::SelectTopicAction::new(app.clone())));

    controllers.register_get_action(Arc::new(super::logs::SelectProcessAction::new()));

    let publisher_controller = super::publisher::PublisherController::new(app.clone());
    controllers.register_post_action(Arc::new(publisher_controller));

    controllers.register_get_action(Arc::new(super::home_controller::IndexAction::new(
        app.clone(),
    )));

    controllers.register_get_action(Arc::new(super::prometheus_controller::MetricsAction::new(
        app.clone(),
    )));

    controllers
}
