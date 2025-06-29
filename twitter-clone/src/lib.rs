pub mod api_response;
pub mod app_state;
pub mod common;
pub mod common_tests;
pub mod error;
pub mod routes;
pub mod schemas;

use std::env;

use actix_web::{http::StatusCode, web, App, HttpServer};
use serde_json::{json, Value};
use tracing_actix_web::TracingLogger;
use tracing_config::init_tracing;

use crate::{
    api_response::ApiResponse,
    common::entities::base::DbRepo,
    error::{IntoClientResult, Result, ServerSideError},
};

pub async fn run() -> Result<()> {
    // Load environment variables
    dotenv::dotenv().ok();

    let port: u16 = env::var("PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .unwrap_or_default();

    let host = env::var("ADDRESS").unwrap_or("127.0.0.1".to_string());

    let _guard = init_tracing();

    let db_repo = DbRepo::init().await;
    let app_data = web::Data::new(app_state::AppState { client: reqwest::Client::new(), db_repo });

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(app_data.clone())
            .service(
                web::scope("/api/v1")
                    .route("/", web::get().to(get_root))
                    .configure(routes::msg_routes::config)
                    .configure(routes::profile_routes::config),
            )
    })
    .bind((host, port))
    .map_err(|err| ServerSideError::HostBindingError(err.to_string()))?
    .run()
    .await
    .map_err(|err| ServerSideError::ServerRunError(err.to_string()))
    .into_client_result()
}

async fn get_root() -> Result<ApiResponse<Value>> {
    let value = json!({
        "message": "Welcome to home page for twitter clone api"
    });
    Ok(ApiResponse::new(StatusCode::OK, value))
}
