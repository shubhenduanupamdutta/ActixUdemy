use actix_multipart::form::{tempfile::TempFile, text::Text, MultipartForm};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ProfileQuery {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ProfileByUserNameQuery {
    pub user_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProfileShort {
    pub id: i64,
    pub user_name: String,
    pub full_name: String,
}

#[derive(Debug, MultipartForm)]
pub struct ProfileCreateMultipart {
    pub user_name: Text<String>,
    pub full_name: Text<String>,
    pub description: Text<String>,
    pub region: Option<Text<String>>,
    pub main_url: Option<Text<String>>,
    pub avatar: Option<TempFile>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProfileResponder {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub user_name: String,
    pub full_name: String,
    pub description: String,
    pub region: Option<String>,
    pub main_url: Option<String>,
    pub avatar: Option<Vec<u8>>,
}
