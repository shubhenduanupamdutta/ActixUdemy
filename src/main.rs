pub mod error;
pub mod tracing_config;
pub mod utility;

use std::sync::RwLock;

use actix_web::{
    web::{self, Json, Path},
    App, HttpServer,
};
use serde::Deserialize;
use tracing::info;
use tracing_actix_web::TracingLogger;

#[derive(Deserialize)]
struct EntityId {
    id: i64,
}

#[allow(dead_code)]
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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn get_user_name(app_data: web::Data<AppState>, params: Path<EntityId>) -> String {
    let users = app_data.users.read().unwrap();
    users
        .iter()
        .find(|user| user.id == params.id)
        .unwrap()
        .user_name
        .clone()
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
