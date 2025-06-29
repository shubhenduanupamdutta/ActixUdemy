use serde::{Deserialize, Serialize};
use serde_repr::*;
use chrono::prelude::*;
use super::profile::ProfileShort;
use std::vec::Vec;

#[derive(Debug, Deserialize)]
pub struct MessageQuery {
    pub id: i64
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageByFollowingQuery {
    pub follower_id: i64,
    pub last_updated_at: DateTime<Utc>,
    pub page_size: Option<i16>
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessagePostJson {
    pub user_id: i64,
    pub body: String,
    pub group_type: MessageGroupTypes,
    pub broadcasting_msg_id: Option<i64>
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponder {
    pub id: i64,
    pub updated_at: DateTime<Utc>,
    pub body: Option<String>,
    pub likes: i32,
    pub broadcasting_msg: Option<Box<MessageResponder>>,
    pub profile: ProfileShort
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageResponders(pub Vec<MessageResponder>);

#[derive(Debug, Deserialize_repr, Serialize_repr, Clone)]
#[repr(i32)]
pub enum MessageGroupTypes {
    Public = 1,
    Circle = 2
}