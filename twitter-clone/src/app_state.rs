use std::fmt::Debug;

#[derive(Debug)]
pub struct AppState<T: Debug> {
    pub client: reqwest::Client,
    pub db_repo: T,
}
