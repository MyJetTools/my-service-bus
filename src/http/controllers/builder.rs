use std::sync::Arc;

use my_http_server::controllers::ControllersMiddleware;

use crate::app::AppContext;

pub fn build(app: &Arc<AppContext>) -> ControllersMiddleware {
    let mut controllers = ControllersMiddleware::new(None, None);

    controllers.register_post_action(Arc::new(super::topics_controller::CreateTopicAction::new(
        app.clone(),
    )));

    controllers.register_put_action(Arc::new(super::topics_controller::RestoreTopicAction::new(
        app.clone(),
    )));

    controllers.register_get_action(Arc::new(super::topics_controller::GetTopicsAction::new(
        app.clone(),
    )));

    controllers.register_delete_action(Arc::new(super::topics_controller::DeleteTopicAction::new(
        app.clone(),
    )));

    controllers.register_post_action(Arc::new(
        super::topics_controller::UpdatePersistAction::new(app.clone()),
    ));

    controllers.register_delete_action(Arc::new(
        super::sessions_controller::DeleteSessionAction::new(app.clone()),
    ));

    controllers.register_post_action(Arc::new(super::greeting::GreetingAction::new(app.clone())));

    //controllers.register_http_objects(greeting_controller);

    controllers.register_post_action(Arc::new(super::greeting::PingAction::new(app.clone())));

    controllers.register_get_action(Arc::new(super::status_controller::GetStatusAction::new(
        app.clone(),
    )));

    controllers.register_get_action(Arc::new(
        super::status_controller::GetStatusLegacyAction::new(app.clone()),
    ));

    controllers.register_get_action(Arc::new(super::queues::GetQueuesAction::new(app.clone())));
    controllers.register_post_action(Arc::new(super::queues::SetMessageIdAction::new(
        app.clone(),
    )));

    controllers.register_post_action(Arc::new(super::queues::SetMaxMessagePerPayloadAction::new(
        app.clone(),
    )));

    controllers
        .register_delete_action(Arc::new(super::queues::DeleteQueueAction::new(app.clone())));

    // DEBUG

    controllers.register_get_action(Arc::new(
        super::debug_controller::GetMinMessageIdAction::new(app.clone()),
    ));

    controllers.register_get_action(Arc::new(
        super::debug_controller::GetQueuesAwaitingToDeliverAction::new(app.clone()),
    ));

    let on_delivery_controller =
        Arc::new(super::debug_controller::OnDeliveryAction::new(app.clone()));
    controllers.register_get_action(on_delivery_controller);

    /*
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
    */

    controllers.register_post_action(Arc::new(super::publisher::PublishAction::new(app.clone())));

    controllers.register_get_action(Arc::new(super::home_controller::IndexAction::new(
        app.clone(),
    )));

    controllers.register_get_action(Arc::new(super::prometheus_controller::MetricsAction::new(
        app.clone(),
    )));

    //Subscribers

    controllers.register_post_action(Arc::new(
        super::subscribers_controller::SubscribeAction::new(app.clone()),
    ));

    controllers.register_post_action(Arc::new(
        super::subscribers_controller::ConfirmDeliveryAction::new(app.clone()),
    ));

    controllers.register_post_action(Arc::new(
        super::subscribers_controller::AwaitDeliveryAction::new(app.clone()),
    ));

    controllers
}
