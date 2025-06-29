pub mod api_response;
pub mod common;
pub mod error;

use std::env;

use actix_web::{web, App, HttpServer};
use serde_json::{json, Value};
use tracing_actix_web::TracingLogger;
use tracing_config::init_tracing;

use crate::{
    api_response::ApiResponse,
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

    HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/", web::get().to(get_root))
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
    Ok(ApiResponse::ok(value))
}
