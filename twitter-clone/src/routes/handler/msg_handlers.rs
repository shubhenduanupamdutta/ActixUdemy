use crate::error::{IntoClientResult, Result};
use crate::{
    api_response::ApiResponse,
    app_state::AppState,
    common::entities::messages::{
        model::MessageQueryResult,
        repo::{InsertMessageFn, QueryMessageFn},
    },
    schemas::message::MessagePostJson,
};
use actix_web::web;
use std::fmt::Debug;
use tracing::{info, instrument};

#[instrument(skip(app_data))]
pub(crate) async fn create_message<T: Debug + InsertMessageFn>(
    app_data: web::Data<AppState<T>>,
    msg: web::Json<MessagePostJson>,
) -> Result<ApiResponse<i64>> {
    info!("Create message handler called");
    let max_size = 281;
    let body = &msg.body[..max_size.min(msg.body.len())];

    let group_type = msg.group_type.clone() as i32;

    let result = app_data
        .db_repo
        .insert_message(msg.user_id, body, group_type, msg.broadcasting_msg_id)
        .await?;
    Ok(ApiResponse::created(result))
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
