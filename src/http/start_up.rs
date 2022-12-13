use std::{net::SocketAddr, sync::Arc};

use my_http_server::{MyHttpServer, StaticFilesMiddleware};

use my_http_server_controllers::swagger::SwaggerMiddleware;

use crate::app::AppContext;

pub fn setup_server(app: Arc<AppContext>) {
    let mut http_server = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 6123)));

    let controllers = Arc::new(crate::http::controllers::builder::build(app.clone()));

    let swagger_middleware = SwaggerMiddleware::new(
        controllers.clone(),
        "MyServiceBus".to_string(),
        crate::app::APP_VERSION.to_string(),
    );

    http_server.add_middleware(Arc::new(swagger_middleware));
    http_server.add_middleware(controllers);

    http_server.add_middleware(Arc::new(StaticFilesMiddleware::new(None, None)));
    http_server.start(app.states.clone(), crate::LOGS.clone());
}
