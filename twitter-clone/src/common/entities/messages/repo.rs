use super::model::MessageWithFollowingAndBroadcastQueryResult;
use crate::common::entities::base::{DbConnGetter, DbRepo, EntityId};
use crate::common::entities::messages::model::MessageWithProfileQueryResult;
use crate::error::{IntoClientResult, Result, ServerSideError};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mockall::automock;
use sqlx::{Pool, Postgres};
use tracing::error;
// 1. we create a single logical container where multiple related members can exist
// 2. we create repeatable structure to our code
// 3. we can hide some members even from our parent module
mod private_members {

    use tracing::instrument;

    use super::*;

    #[instrument(skip())]
    pub(crate) async fn insert_message_inner(
        conn: &Pool<Postgres>,
        user_id: i64,
        body: &str,
        group_type: i32,
        broadcasting_msg_id: Option<i64>,
    ) -> Result<i64> {
        let mut tx = conn.begin().await.map_err(ServerSideError::from)?;

        let insert_msg_result = sqlx::query_as::<_, EntityId>(
            "insert into message (user_id, body, msg_group_type) values ($1, $2, $3) returning id",
        )
        .bind(user_id)
        .bind(body)
        .bind(group_type)
        .fetch_one(&mut *tx)
        .await;

        let message_id_result = match insert_msg_result {
            Ok(r) => Ok(r.id),
            Err(e) => {
                error!("insert_message error: {}", e);
                return Err(ServerSideError::from(e).into());
            },
        };

        if let Some(bm_id) = broadcasting_msg_id {
            let message_broadcast_result = sqlx
                ::query_as::<_, EntityId>(
                    "insert into message_broadcast (main_msg_id, broadcasting_msg_id) values ($1, $2) returning id"
                )
                .bind(message_id_result.as_ref().unwrap())
                .bind(bm_id)
                .fetch_one(&mut *tx).await;

            if message_broadcast_result.is_err() {
                _ = tx.rollback().await;
                return Err(ServerSideError::from(message_broadcast_result.err().unwrap()).into());
            }
        }

        _ = tx.commit().await;

        message_id_result.into_client_result()
    }

    #[instrument(skip())]
    pub(crate) async fn insert_response_message_inner(
        conn: &Pool<Postgres>,
        user_id: i64,
        body: &str,
        group_type: i32,
        original_msg_id: i64,
    ) -> Result<i64> {
        let mut tx = conn.begin().await.unwrap();

        let insert_result = sqlx::query_as::<_, EntityId>(
            "insert into message (user_id, body, msg_group_type) values ($1, $2, $3) returning id",
        )
        .bind(user_id)
        .bind(body)
        .bind(group_type)
        .fetch_one(&mut *tx)
        .await;

        let msg_id = match insert_result {
            Ok(r) => r.id,
            Err(e) => {
                error!("insert_message error: {}", e);
                return Err(ServerSideError::from(e).into());
            },
        };

        let insert_msg_response_result = sqlx
            ::query_as::<_, EntityId>(
                "insert into message_response (original_msg_id, responding_msg_id) values ($1, $2) returning id"
            )
            .bind(original_msg_id)
            .bind(msg_id)
            .fetch_one(&mut *tx).await;

        match insert_msg_response_result {
            Ok(_) => {
                tx.commit().await.map_err(ServerSideError::from)?;
                return Ok(msg_id);
            },
            Err(e) => {
                tx.rollback().await.map_err(ServerSideError::from)?;
                return Err(ServerSideError::from(e).into());
            },
        }
    }

    #[instrument(skip())]
    pub(crate) async fn query_message_inner(
        conn: &Pool<Postgres>,
        id: i64,
    ) -> Result<Option<MessageWithFollowingAndBroadcastQueryResult>> {
        let message_result = sqlx
            ::query_as::<_, MessageWithProfileQueryResult>(
                r"
                select m.id, m.updated_at, m.body, m.likes, m.image, m.msg_group_type, m.user_id, p.user_name, p.full_name, p.avatar, mb.id as broadcast_msg_id
                    from message m
                        join profile p on m.user_id = p.id
                        left join message_broadcast mb on m.id = mb.main_msg_id
                    where
                        m.id = $1
            "
            )
            .bind(id)
            .fetch_optional(conn).await;

        match message_result {
            Ok(message) => {
                if let Some(msg) = message {
                    let optional_matching_broadcast_message =
                        get_broadcasting_message_of_message(conn, &msg).await;
                    let final_message = append_broadcast_msg_to_msg(
                        optional_matching_broadcast_message.as_ref(),
                        &msg,
                    );
                    Ok(Some(final_message))
                } else {
                    Ok(None)
                }
            },
            Err(e) => Err(ServerSideError::from(e).into()),
        }
    }

    #[instrument(skip())]
    pub(crate) async fn query_messages_inner(
        conn: &Pool<Postgres>,
        user_id: i64,
        last_updated_at: DateTime<Utc>,
        page_size: i16,
    ) -> Result<Vec<MessageWithFollowingAndBroadcastQueryResult>> {
        let following_messages_with_profiles_result = sqlx
            ::query_as::<_, MessageWithProfileQueryResult>(
                r"
                select m.id, m.updated_at, m.body, m.likes, m.image, m.msg_group_type, m.user_id, p.user_name, p.full_name, p.avatar, mb.id as broadcast_msg_id
                    from message m
                        join follow f on m.user_id = f.following_id
                        join profile p on p.id = f.following_id
                        left join message_broadcast mb on m.id = mb.main_msg_id
                        where
                            f.follower_id = $1
                            and m.updated_at < $2
                        order by m.updated_at desc
                        limit $3
            "
            )
            .bind(user_id)
            .bind(last_updated_at)
            .bind(page_size)
            .fetch_all(conn).await;

        match following_messages_with_profiles_result {
            Ok(following_messages) => {
                let following_messages_with_broadcasts = following_messages
                    .clone()
                    .into_iter()
                    .filter(|msg| {
                        msg.broadcast_msg_id.is_some() && msg.broadcast_msg_id.unwrap() > 0
                    })
                    .collect::<Vec<MessageWithProfileQueryResult>>();

                let optional_matching_broadcast_messages = get_broadcasting_messages_of_messages(
                    conn,
                    &following_messages_with_broadcasts,
                )
                .await;
                let final_message_list = append_broadcast_msgs_to_msgs(
                    &optional_matching_broadcast_messages,
                    following_messages,
                );
                Ok(final_message_list)
            },
            Err(e) => Err(ServerSideError::from(e).into()),
        }
    }

    #[instrument(skip())]
    async fn get_broadcasting_messages_of_messages(
        conn: &Pool<Postgres>,
        following_messages_with_broadcasts: &Vec<MessageWithProfileQueryResult>,
    ) -> Option<Vec<MessageWithProfileQueryResult>> {
        let following_broadcast_message_ids = following_messages_with_broadcasts
            .iter()
            .map(|msg| msg.broadcast_msg_id.unwrap())
            .collect::<Vec<i64>>();

        let broadcasting_msg_result = sqlx
            ::query_as::<_, MessageWithProfileQueryResult>(
                r"
                select m.id, m.updated_at, m.body, m.likes, m.image, m.msg_group_type, m.user_id, p.user_name, p.full_name, p.avatar, mb.id as broadcast_msg_id
                    from message m
                        join profile p on m.user_id = p.id
                        left join message_broadcast mb on m.id = mb.main_msg_id
                    where m.id = ANY($1)
            "
            )
            .bind(following_broadcast_message_ids)
            .fetch_all(conn).await;

        match broadcasting_msg_result {
            Ok(broadcast_messages) => Some(broadcast_messages),
            Err(e) => {
                println!("get_broadcasting_messages_of_messages: {}", e);
                None
            },
        }
    }

    #[instrument(skip())]
    async fn get_broadcasting_message_of_message(
        conn: &Pool<Postgres>,
        message: &MessageWithProfileQueryResult,
    ) -> Option<MessageWithProfileQueryResult> {
        let broadcasting_msg_result = sqlx
            ::query_as::<_, MessageWithProfileQueryResult>(
                r"
                select m.id, m.updated_at, m.body, m.likes, m.image, m.msg_group_type, m.user_id, p.user_name, p.full_name, p.avatar, mb.id as broadcast_msg_id
                    from message m
                        join profile p on m.user_id = p.id
                        left join message_broadcast mb on m.id = mb.main_msg_id
                    where m.id = $1
            "
            )
            .bind(message.broadcast_msg_id)
            .fetch_optional(conn).await;

        match broadcasting_msg_result {
            Ok(broadcast_message) => broadcast_message,
            Err(e) => {
                println!("get_broadcasting_messages_of_messages: {}", e);
                None
            },
        }
    }

    #[instrument(skip())]
    fn append_broadcast_msgs_to_msgs(
        optional_broadcast_messages: &Option<Vec<MessageWithProfileQueryResult>>,
        following_messages_with_broadcasts: Vec<MessageWithProfileQueryResult>,
    ) -> Vec<MessageWithFollowingAndBroadcastQueryResult> {
        let mut final_list_of_messages: Vec<MessageWithFollowingAndBroadcastQueryResult> = vec![];

        following_messages_with_broadcasts
            .iter()
            .for_each(|following_message_with_broadcast| {
                let matching_broadcast_msg =
                    if let Some(broadcast_messages) = optional_broadcast_messages {
                        broadcast_messages.iter().find(|bm| {
                            Some(bm.id) == following_message_with_broadcast.broadcast_msg_id
                        })
                    } else {
                        None
                    };

                final_list_of_messages.push(append_broadcast_msg_to_msg(
                    matching_broadcast_msg,
                    following_message_with_broadcast,
                ));
            });

        final_list_of_messages
    }

    #[instrument(skip())]
    fn append_broadcast_msg_to_msg(
        broadcast_message: Option<&MessageWithProfileQueryResult>,
        message_with_broadcast: &MessageWithProfileQueryResult,
    ) -> MessageWithFollowingAndBroadcastQueryResult {
        let mut final_message = MessageWithFollowingAndBroadcastQueryResult {
            id: message_with_broadcast.id,
            updated_at: message_with_broadcast.updated_at,
            body: message_with_broadcast.body.clone(),
            likes: message_with_broadcast.likes,
            image: message_with_broadcast.image.clone(),
            msg_group_type: message_with_broadcast.msg_group_type,
            user_id: message_with_broadcast.user_id,
            user_name: message_with_broadcast.user_name.clone(),
            full_name: message_with_broadcast.full_name.clone(),
            avatar: message_with_broadcast.avatar.clone(),
            broadcast_msg_id: None,
            broadcast_msg_updated_at: None,
            broadcast_msg_user_id: None,
            broadcast_msg_body: None,
            broadcast_msg_likes: None,
            broadcast_msg_image: None,
            broadcast_msg_user_name: None,
            broadcast_msg_full_name: None,
            broadcast_msg_avatar: None,
        };

        if let Some(matching_broadcast) = broadcast_message {
            final_message.broadcast_msg_id = Some(matching_broadcast.id);
            final_message.broadcast_msg_updated_at = Some(matching_broadcast.updated_at);
            final_message.broadcast_msg_body = matching_broadcast.body.to_owned();
            final_message.broadcast_msg_likes = Some(matching_broadcast.likes);
            final_message.broadcast_msg_image = matching_broadcast.image.to_owned();
            final_message.broadcast_msg_user_id = Some(matching_broadcast.user_id);
            final_message.broadcast_msg_user_name = Some(matching_broadcast.user_name.to_string());
            final_message.broadcast_msg_full_name = Some(matching_broadcast.full_name.to_string());
            final_message.broadcast_msg_avatar = matching_broadcast.avatar.to_owned();
        }

        final_message
    }
}

#[automock]
#[async_trait]
pub trait InsertMessageFn {
    async fn insert_message(
        &self,
        user_id: i64,
        body: &str,
        group_type: i32,
        broadcasting_msg_id: Option<i64>,
    ) -> Result<i64>;
}

#[async_trait]
impl InsertMessageFn for DbRepo {
    async fn insert_message(
        &self,
        user_id: i64,
        body: &str,
        group_type: i32,
        broadcasting_msg_id: Option<i64>,
    ) -> Result<i64> {
        private_members::insert_message_inner(
            self.get_conn(),
            user_id,
            body,
            group_type,
            broadcasting_msg_id,
        )
        .await
    }
}

#[automock]
#[async_trait]
pub trait InsertResponseMessageFn {
    async fn insert_response_message(
        &self,
        user_id: i64,
        body: &str,
        group_type: i32,
        original_msg_id: i64,
    ) -> Result<i64>;
}

#[async_trait]
impl InsertResponseMessageFn for DbRepo {
    async fn insert_response_message(
        &self,
        user_id: i64,
        body: &str,
        group_type: i32,
        original_msg_id: i64,
    ) -> Result<i64> {
        private_members::insert_response_message_inner(
            self.get_conn(),
            user_id,
            body,
            group_type,
            original_msg_id,
        )
        .await
    }
}

#[automock]
#[async_trait]
pub trait QueryMessageFn {
    async fn query_message(
        &self,
        id: i64,
    ) -> Result<Option<MessageWithFollowingAndBroadcastQueryResult>>;
}

#[async_trait]
impl QueryMessageFn for DbRepo {
    async fn query_message(
        &self,
        id: i64,
    ) -> Result<Option<MessageWithFollowingAndBroadcastQueryResult>> {
        private_members::query_message_inner(self.get_conn(), id).await
    }
}

#[automock]
#[async_trait]
pub trait QueryMessagesFn {
    async fn query_messages(
        &self,
        user_id: i64,
        last_updated_at: DateTime<Utc>,
        page_size: i16,
    ) -> Result<Vec<MessageWithFollowingAndBroadcastQueryResult>>;
}

#[async_trait]
impl QueryMessagesFn for DbRepo {
    async fn query_messages(
        &self,
        user_id: i64,
        last_updated_at: DateTime<Utc>,
        page_size: i16,
    ) -> Result<Vec<MessageWithFollowingAndBroadcastQueryResult>> {
        private_members::query_messages_inner(self.get_conn(), user_id, last_updated_at, page_size)
            .await
    }
}
