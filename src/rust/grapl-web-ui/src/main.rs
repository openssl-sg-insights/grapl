mod authn;
mod config;
mod routes;
mod services;

use actix_session::CookieSession;
use actix_web::{
    web,
    web::Data,
    App,
    HttpServer,
};
use actix_web_opentelemetry::RequestTracing;
use opentelemetry::{
    global,
    trace::TraceError,
};

#[derive(thiserror::Error, Debug)]
enum GraplUiError {
    #[error(transparent)]
    Config(#[from] config::ConfigError),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Trace Error: {0}")]
    Trace(#[from] TraceError),
}

#[actix_web::main]
async fn main() -> Result<(), GraplUiError> {
    let (_env, _guard) = grapl_config::init_grapl_env!("web-ui");

    let config = config::Config::from_env()?;

    let bind_address = config.bind_address.clone();

    HttpServer::new(move || {
        let web_client = Data::new(awc::Client::new());
        let auth_dynamodb_client = Data::new(authn::AuthDynamoClient {
            client: config.dynamodb_client.clone(),
            user_auth_table_name: config.user_auth_table_name.clone(),
            user_session_table_name: config.user_session_table_name.clone(),
        });
        let graphql_endpoint = Data::new(config.graphql_endpoint.clone());
        let model_plugin_deployer_endpoint =
            Data::new(config.model_plugin_deployer_endpoint.clone());

        App::new()
            .wrap(actix_web::middleware::Logger::default())
            .wrap(RequestTracing::new())
            .wrap(actix_web::middleware::Compress::default())
            .wrap(
                CookieSession::private(&config.session_key)
                    .path("/")
                    .secure(true),
            )
            .app_data(web_client)
            .app_data(graphql_endpoint)
            .app_data(model_plugin_deployer_endpoint)
            .app_data(auth_dynamodb_client)
            .configure(routes::config)
            .service(web::scope("/graphQlEndpoint").configure(services::graphql::config))
            .service(
                web::scope("/modelPluginDeployer")
                    .configure(services::model_plugin_deployer::config),
            )
    })
    .bind(bind_address)?
    .run()
    .await?;

    // sending remaining trace spans.
    global::shutdown_tracer_provider();

    Ok(())
}
