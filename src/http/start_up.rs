use std::{net::SocketAddr, sync::Arc};

use my_http_server::{HttpConnectionsCounter, MyHttpServer, StaticFilesMiddleware};

use my_http_server::controllers::swagger::SwaggerMiddleware;

use crate::app::AppContext;

use super::auth::AuthMiddleware;

pub fn setup_server(app: &Arc<AppContext>) -> HttpConnectionsCounter {
    let mut http_server = MyHttpServer::new(SocketAddr::from(([0, 0, 0, 0], 6123)));

    let controllers = Arc::new(crate::http::controllers::builder::build(app));

    let swagger_middleware = SwaggerMiddleware::new(
        controllers.clone(),
        "MyServiceBus".to_string(),
        crate::app::APP_VERSION.to_string(),
    );

    http_server.add_middleware(Arc::new(swagger_middleware));

    http_server.add_middleware(Arc::new(AuthMiddleware));

    http_server.add_middleware(controllers);

    http_server.add_middleware(Arc::new(StaticFilesMiddleware::new(None, None)));
    http_server.start(app.states.clone(), app.logs.clone());

    http_server.get_http_connections_counter()
}
