use chrono::{DateTime, Utc};
use fake::{
    faker::{
        internet::en::Username,
        name::en::{FirstName, LastName},
    },
    Fake,
};
use serde::Deserialize;
use sqlx::{postgres, query_as, FromRow};

#[tokio::main]
pub async fn main() {
    let conn = postgres::PgPool::connect("postgres://tester:tester@localhost:5432/tester")
        .await
        .unwrap();

    // let result = query_as::<_, Profile>("select * from profile where id = $1")
    //     .bind(2)
    //     .fetch_one(&conn)
    //     .await;
    // println!("Get statement result: {:?}", result.unwrap());

    // // Insert statement
    // let id = query_as::<_, EntityId>(
    //     "insert into message (user_id, body, likes) values ($1, $2, $3) returning id",
    // )
    // .bind(2)
    // .bind("Hello World!")
    // .bind(10)
    // .fetch_one(&conn)
    // .await;
    // println!("Insert statement result: {:?}", id);

    // A transaction
    let mut tx = conn.begin().await.unwrap();
    let user_id = query_as::<_, EntityId>(
        "insert into profile (user_name, full_name) values ($1, $2) returning id",
    )
    .bind(Username().fake::<String>())
    .bind(format!(
        "{} {}",
        FirstName().fake::<String>(),
        LastName().fake::<String>()
    ))
    .fetch_one(&mut *tx)
    .await
    .unwrap()
    .id;

    println!("Insert result inside transaction: {:?}", user_id);

    let query_result = query_as::<_, Profile>("select * from profile where id = $1")
        .bind(user_id)
        .fetch_one(&mut *tx)
        .await;

    match query_result {
        Ok(profile) => {
            println!("Query Result after transaction: {:?}", profile);
            _ = tx.commit().await;
        },
        Err(e) => {
            println!("Query result after transaction Failed: {}", e.to_string());
            _ = tx.rollback().await;
        },
    }
}

#[allow(dead_code)]
#[derive(Debug, FromRow, Deserialize)]
struct Profile {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub user_name: String,
    pub full_name: String,
}

#[derive(Debug, Deserialize, FromRow)]
struct EntityId {
    pub id: i64,
}
