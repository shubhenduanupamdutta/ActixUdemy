# Code Along for the Rust Actix course on Udemy [Build API Servers with Rust and Actix](https://www.udemy.com/course/learn-rust-actix-web-and-sqlx/)

## This is a demo of api clone of twitter

---

## NOTE ON `HttpServer` Setup

**When we setup our server, although we set only one time, but there is going to be multiple instance set, default behavior is going to be one instance per core. If we want truly global state across all the individual threads, then we actually instantiate the state data, and then we need to clone it. Behind the scene, actix automatically wraps the data into an `Arc`. Cloning makes sure that each of the thread are the owner of real data.**

---

## Extractor

### What is an extractor?

The parameters which are going to be used in our actix app, like getting post body, globally shared
app state, which is all being inserted via parameters, are basically extract.
Furthermore, although the data is in string format or json body, properly set extractors make sure
that they are transformed into statically typed rust types.

### Basic Path, AppState and Json Extractor Example

```rust
use std::sync::RwLock;

use actix_web::{
    middleware::Logger,
    web::{self, Json, Path},
    App, HttpServer,
};
use serde::Deserialize;

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

    // Initializing logger for logging
    env_logger::init();

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

    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .wrap(Logger::default())
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
```

### `FromRequest` trait

Those built in capabilities of extractors to take input in raw format and convert it into Rust types, are based on the implementation of `FromRequest` trait. So any custom extractors we need to create must also implement this trait.

---

## Responders

Taking static Rust types and convert them to a format which we can send to client.
`Responder` is a trait that has capability and methods of converting from rust type to an output type.
We can use any type that implements this trait, like `HttpResponse`, `Json<T>` and and Custom Type which has `Responder` trait, can be used as a response (output from a route).

## Using `Either` to redirect if needed

```rust
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
```

## `sqlx` crate

### Building a container with postgres

// docker-compose.yaml

```yaml
version: "3.8"

services:
  db:
    image: postgres:14-alpine
    ports:
      - 5432:5432
    environment:
      POSTGRES_PASSWORD: tester
      POSTGRES_USER: tester
      POSTGRES_DB: tester
    volumes:
      - ./dbdata:/var/lib/postgresql/data
```

Then in the folder run

```bash
docker compose up -d --build
```

### Add sqlx dependencies

```toml
sqlx = { version = "0.8.6", features = [
    "postgres",
    "runtime-tokio",
    "tls-native-tls",
    "chrono",
] }
thiserror = "2.0.12"
tokio = { version = "1.45.1", features = ["full"] }
```

### Install sqlx-cli

```bash
cargo install sqlx-cli
```

### Workflow

#### First create the database

```bash
sqlx database create --database-url postgres://tester:tester@localhost:5432/tester
```

#### Test that database is created

```bash
psql -h localhost -p 5432 -d tester -U tester
```

#### Creating a new migration file (empty)

```bash
sqlx migrate add profile_table
```

#### Running the migration

```bash
sqlx migrate run --database-url postgres://tester:tester@localhost:5432/tester
```
