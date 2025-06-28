pub mod error;
pub mod tracing_config;
pub mod utility;
pub mod sqlx_example;

use error::Result;
use serde_json::{json, Value};
use std::sync::RwLock;

use actix_web::{
    http::StatusCode,
    web::{self, Json, Path, Redirect},
    App, Either, HttpServer, Responder,
};
use serde::{Deserialize, Serialize};
use tracing::info;
use tracing_actix_web::TracingLogger;

use crate::{error::ServerSideError, utility::api_response::ApiResponse};

#[derive(Deserialize)]
struct EntityId {
    id: i64,
}

#[derive(Clone, Serialize)]
struct FinalUser {
    id: i64,
    user_name: String,
    full_name: String,
}

#[derive(Deserialize)]
struct NewUser {
    user_name: String,
    full_name: String,
}

struct AppState {
    users: RwLock<Vec<FinalUser>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initializing dotenv to load environment variables
    dotenv::dotenv().ok();

    let _guards = tracing_config::init_tracing();

    sqlx_example::example().await;

    let app_data = web::Data::new(AppState {
        users: RwLock::new(vec![
            FinalUser {
                id: 1,
                user_name: "dave".to_string(),
                full_name: "Dave Choi".to_string(),
            },
            FinalUser {
                id: 2,
                user_name: "linda".to_string(),
                full_name: "John Strong".to_string(),
            },
            FinalUser {
                id: 3,
                user_name: "john".to_string(),
                full_name: "John Strong".to_string(),
            },
        ]),
    });

    info!("Starting Actix Web Server...");

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .wrap(TracingLogger::default())
            .service(
                web::scope("/v1")
                    .service(web::resource("/user/{id}").route(web::get().to(get_user_name)))
                    .service(web::resource("/user").route(web::post().to(insert_user))),
            )
            .service(web::resource("/na").route(web::get().to(failure_message)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn get_user_name(
    app_data: web::Data<AppState>,
    params: Path<EntityId>,
) -> Either<Result<impl Responder>, Result<ApiResponse<FinalUser>>> {
    let users = app_data.users.read().unwrap();
    let user = users.iter().find(|user| user.id == params.id);

    match user {
        Some(user) if user.id != 3 => Either::Left(Ok(Redirect::new("/", "../../na"))),
        Some(user) => Either::Right(Ok(ApiResponse::ok(user.clone()))),
        None => Either::Right(Err(ServerSideError::InternalServerError(
            "Internal Server Error".to_string(),
        )
        .into())),
    }
}

async fn insert_user(app_data: web::Data<AppState>, new_user: Json<NewUser>) -> String {
    let mut users = app_data.users.write().unwrap();
    let max_id = users.iter().max_by_key(|user| user.id).unwrap().id;
    users.push(FinalUser {
        id: max_id + 1,
        user_name: new_user.user_name.clone(),
        full_name: new_user.full_name.clone(),
    });

    users.last().unwrap().id.to_string()
}

async fn failure_message() -> ApiResponse<Value> {
    let value = json!({
        "error": "Some unknown error has occurred."
    });

    ApiResponse::new(StatusCode::INTERNAL_SERVER_ERROR, value)
}
