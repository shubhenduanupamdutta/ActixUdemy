use crate::common::entities::base::{DbConnGetter, DbRepo};
use crate::common::entities::{base::EntityId, profile::model::ProfileCreate};
use crate::error::Result;
use async_trait::async_trait;
use mockall::automock;
use sqlx::{Pool, Postgres};
use tracing::{error, instrument};

use crate::{
    common::entities::profile::model::ProfileQueryResult,
    error::{IntoClientResult, ServerSideError},
};

mod private_members {

    use super::*;

    #[instrument(skip())]
    pub(crate) async fn insert_profile_inner(
        conn: &Pool<Postgres>,
        params: ProfileCreate,
    ) -> Result<i64> {
        let result = sqlx::query_as::<_, EntityId>(
            r"
            insert into Profile
                (user_name, full_name, description, region, main_url, avatar)
                values
                ($1, $2, $3, $4, $5, $6)
            returning id",
        )
        .bind(&params.user_name)
        .bind(&params.full_name)
        .bind(&params.description)
        .bind(&params.region)
        .bind(&params.main_url)
        .bind(&params.avatar)
        .fetch_one(conn)
        .await;

        let result = match result {
            Ok(r) => Ok(r.id),
            Err(e) => {
                error!("Failed to insert profile: {:?}", e);
                Err(ServerSideError::from(e))
            },
        };
        result.into_client_result()
    }

    #[instrument(skip())]
    pub(crate) async fn update_profile_avatar_inner(
        conn: &Pool<Postgres>,
        profile_id: i64,
        avatar: Vec<u8>,
    ) -> Result<()> {
        sqlx::query::<_>(
            r"
            update Profile
                set avatar = $1 where id = $2
            ",
        )
        .bind(avatar)
        .bind(profile_id)
        .execute(conn)
        .await
        .map(|_| ())
        .map_err(|e| {
            error!("Failed to update profile avatar: {:?}", e);
            ServerSideError::from(e)
        })
        .into_client_result()
    }

    #[instrument(skip())]
    pub(crate) async fn follow_user_inner(
        conn: &Pool<Postgres>,
        follower_id: i64,
        following_id: i64,
    ) -> Result<i64> {
        sqlx::query_as::<_, EntityId>(
            "insert into follow (follower_id, following_id) values ($1, $2) returning id",
        )
        .bind(follower_id)
        .bind(following_id)
        .fetch_one(conn)
        .await
        .map(|row: EntityId| row.id)
        .map_err(|e| {
            error!("Failed to follow user: {:?}", e);
            ServerSideError::from(e)
        })
        .into_client_result()
    }

    #[instrument(skip())]
    pub(crate) async fn query_profile_inner(
        conn: &Pool<Postgres>,
        id: i64,
    ) -> Result<Option<ProfileQueryResult>> {
        sqlx::query_as::<_, ProfileQueryResult>("select * from profile where id = $1")
            .bind(id)
            .fetch_optional(conn)
            .await
            .map_err(ServerSideError::from)
            .into_client_result()
    }

    #[instrument(skip())]
    pub(crate) async fn query_profile_by_user_inner(
        conn: &Pool<Postgres>,
        user_name: String,
    ) -> Result<Option<ProfileQueryResult>> {
        sqlx::query_as::<_, ProfileQueryResult>("select * from profile where user_name = $1")
            .bind(user_name)
            .fetch_optional(conn)
            .await
            .map_err(ServerSideError::from)
            .into_client_result()
    }
}

#[automock]
#[async_trait]
pub trait InsertProfileFn {
    async fn insert_profile(&self, params: ProfileCreate) -> Result<i64>;
}

#[async_trait]
impl InsertProfileFn for DbRepo {
    async fn insert_profile(&self, params: ProfileCreate) -> Result<i64> {
        private_members::insert_profile_inner(self.get_conn(), params).await
    }
}

#[automock]
#[async_trait]
pub trait UpdateProfileAvatarFn {
    async fn update_profile_avatar(&self, user_id: i64, avatar: Vec<u8>) -> Result<()>;
}

#[async_trait]
impl UpdateProfileAvatarFn for DbRepo {
    async fn update_profile_avatar(&self, user_id: i64, avatar: Vec<u8>) -> Result<()> {
        private_members::update_profile_avatar_inner(self.get_conn(), user_id, avatar).await
    }
}

#[automock]
#[async_trait]
pub trait QueryProfileFn {
    async fn query_profile(&self, id: i64) -> Result<Option<ProfileQueryResult>>;
}

#[async_trait]
impl QueryProfileFn for DbRepo {
    async fn query_profile(&self, id: i64) -> Result<Option<ProfileQueryResult>> {
        private_members::query_profile_inner(self.get_conn(), id).await
    }
}

#[automock]
#[async_trait]
pub trait QueryProfileByUserFn {
    async fn query_profile_by_user(&self, user_name: String) -> Result<Option<ProfileQueryResult>>;
}

#[async_trait]
impl QueryProfileByUserFn for DbRepo {
    async fn query_profile_by_user(&self, user_name: String) -> Result<Option<ProfileQueryResult>> {
        private_members::query_profile_by_user_inner(self.get_conn(), user_name).await
    }
}

#[automock]
#[async_trait]
pub trait FollowUserFn {
    async fn follow_user(&self, follower_id: i64, following_id: i64) -> Result<i64>;
}

#[async_trait]
impl FollowUserFn for DbRepo {
    async fn follow_user(&self, follower_id: i64, following_id: i64) -> Result<i64> {
        private_members::follow_user_inner(self.get_conn(), follower_id, following_id).await
    }
}
