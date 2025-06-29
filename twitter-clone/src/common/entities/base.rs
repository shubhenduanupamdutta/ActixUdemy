use std::env;

use serde::Deserialize;
use sqlx::{migrate, prelude::FromRow, Pool, Postgres};
use tracing::{error, info};

#[derive(Debug, FromRow, Deserialize)]
pub struct EntityId {
    pub id: i64,
}

/// This trait allows us to decouple getting connection and then we could use it with real database
/// or mock database as needed. This also allows us to restrict access to the conn to other users
pub trait DbConnGetter {
    type Output;
    fn get_conn(&self) -> &Self::Output;
}

#[derive(Debug, Clone)]
pub struct DbRepo {
    conn: Pool<Postgres>,
}

impl DbRepo {
    pub async fn init() -> Self {
        Self { conn: get_db_conn().await }
    }
}

impl DbConnGetter for DbRepo {
    type Output = Pool<Postgres>;

    fn get_conn(&self) -> &Self::Output {
        &self.conn
    }
}

pub async fn get_db_conn() -> Pool<Postgres> {
    dotenv::dotenv().ok();

    let postgres_host = env::var("POSTGRES_HOST").unwrap();
    let postgres_port = env::var("POSTGRES_PORT")
        .unwrap_or("5432".to_string())
        .parse::<u16>()
        .unwrap_or_default();
    let postgres_password =
        env::var("POSTGRES_PASSWORD").expect("There should be a valid password for database");
    let postgres_user =
        env::var("POSTGRES_USER").expect("There should be a valid user defined for database.");
    let postgres_db =
        env::var("POSTGRES_DB").expect("There should be valid postgres database defined.");

    let postgres_url = format!("postgres://{postgres_user}:{postgres_password}@{postgres_host}:{postgres_port}/{postgres_db}");

    let conn = sqlx::postgres::PgPool::connect(&postgres_url)
        .await
        .expect("Connection to database shouldn't fail.");
    let migrate = migrate!("./migrations").run(&conn).await;

    match migrate {
        Ok(()) => info!("sqlx migration success"),
        Err(e) => error!("Sqlx migration error: {:?}", e),
    }
    conn
}
