use crate::common::entities::base::DbRepo;
use crate::common::entities::messages::model::MessageWithFollowingAndBroadcastQueryResult;
use crate::error::{Result, ServerSideError};
use crate::schemas::message::{MessageByFollowingQuery, MessageResponder, MessageResponders};
use crate::schemas::profile::ProfileShort;
use crate::{
    api_response::ApiResponse,
    app_state::AppState,
    common::entities::messages::repo::{InsertMessageFn, QueryMessageFn, QueryMessagesFn},
    schemas::message::MessagePostJson,
};
use actix_web::web;
use serde_json::{json, Value};
use std::fmt::Debug;
use tracing::{info, instrument};

#[instrument(skip(app_data))]
pub(crate) async fn create_message<T: Debug + InsertMessageFn>(
    app_data: web::Data<AppState<T>>,
    msg: web::Json<MessagePostJson>,
) -> Result<ApiResponse<Value>> {
    info!("Create message handler called");
    let max_size = 281;
    let body = &msg.body[..max_size.min(msg.body.len())];

    let group_type = msg.group_type.clone() as i32;

    let result = app_data
        .db_repo
        .insert_message(msg.user_id, body, group_type, msg.broadcasting_msg_id)
        .await?;
    info!("Message created with id: {}", result);
    Ok(ApiResponse::created(json!({
        "message": "Message created successfully",
        "message_id": result
    })))
}

#[instrument(skip(app_data))]
pub(crate) async fn get_message<T: Debug + QueryMessageFn>(
    app_data: web::Data<AppState<DbRepo>>,
    path: web::Path<i64>,
) -> Result<ApiResponse<MessageResponder>> {
    info!("Get message handler called for id: {}", path);
    let message_id = path.into_inner();
    let message = app_data.db_repo.query_message(message_id).await?;
    if message.is_none() {
        return Err(ServerSideError::MessageNotFound(format!(
            "No message found with id: {message_id}"
        ))
        .into());
    }
    Ok(ApiResponse::ok(message.unwrap().into()))
}

#[instrument(skip(app_data))]
pub(crate) async fn get_messages<T: Debug + QueryMessagesFn>(
    app_data: web::Data<AppState<T>>,
    path: web::Json<MessageByFollowingQuery>,
) -> Result<ApiResponse<MessageResponders>> {
    info!(
        "Get messages handler called for follower_id: {}",
        path.follower_id
    );

    let page_size = path.page_size.unwrap_or(10);

    let messages = app_data
        .db_repo
        .query_messages(path.follower_id, path.last_updated_at, page_size)
        .await?;
    info!("Fetched {} messages", messages.len());

    let msg_collection: Vec<MessageResponder> =
        messages.into_iter().map(MessageResponder::from).collect();

    Ok(ApiResponse::ok(MessageResponders(msg_collection)))
}

impl From<MessageWithFollowingAndBroadcastQueryResult> for MessageResponder {
    fn from(message: MessageWithFollowingAndBroadcastQueryResult) -> Self {
        MessageResponder {
            id: message.id,
            updated_at: message.updated_at,
            body: message.body.clone(),
            likes: message.likes,
            broadcasting_msg: match message.broadcast_msg_id {
                Some(id) => Some(Box::new(MessageResponder {
                    id,
                    updated_at: message.broadcast_msg_updated_at.unwrap(),
                    body: message.broadcast_msg_body.clone(),
                    likes: message.broadcast_msg_likes.unwrap(),
                    broadcasting_msg: None,
                    profile: ProfileShort {
                        id: message.broadcast_msg_user_id.unwrap(),
                        user_name: message.broadcast_msg_user_name.clone().unwrap(),
                        full_name: message.broadcast_msg_full_name.clone().unwrap(),
                    },
                })),
                None => None,
            },
            profile: ProfileShort {
                id: message.id,
                user_name: message.user_name.clone(),
                full_name: message.full_name.clone(),
            },
        }
    }
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;
    use crate::common::entities::messages::repo::InsertMessageFn;
    use crate::common_tests::get_app_data;
    use crate::schemas::message::MessageGroupTypes;
    use std::fmt::Debug;

    mod test_success_from_create_message {
        use super::*;

        #[derive(Debug)]
        struct MockRepo;
        #[async_trait::async_trait]
        impl InsertMessageFn for MockRepo {
            async fn insert_message(
                &self,
                user_id: i64,
                body: &str,
                group_type: i32,
                broadcasting_msg_id: Option<i64>,
            ) -> Result<i64> {
                Ok(42)
            }
        }

        #[tokio::test]
        async fn test_create_message_normal_body() {
            let repo = MockRepo;
            let app_data = get_app_data(repo).await;
            let msg = MessagePostJson {
                user_id: 1,
                body: "Hello, world!".to_string(),
                group_type: MessageGroupTypes::Public,
                broadcasting_msg_id: None,
            };
            let result = create_message(app_data, web::Json(msg)).await.unwrap();
            assert_eq!(result.data, 42);
        }
    }

    // /// Create failure returns correct error
    // #[tokio::test]
    // async fn test_create_message_failure() {
    //     let repo = MockRepo;
    //     let app_data = get_app_data(repo).await;
    //     let msg = MessagePostJson {
    //         user_id: 1,
    //         body: "Hello, world!".to_string(),
    //         group_type: MessageGroupTypes::Public,
    //         broadcasting_msg_id: None,
    //     };
    //     // Simulate a failure by modifying the MockRepo to return an error
    //     let result = create_message(app_data, web::Json(msg)).await;
    //     assert!(result.is_err());
    // }
}
