use twitter_clone::{error::Result, run};

#[actix_web::main]
async fn main() -> Result<()> {
    run().await
}
