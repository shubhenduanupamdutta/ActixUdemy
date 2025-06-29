use crate::common::entities::profile::model::{ProfileCreate, ProfileQueryResult};
use crate::common::entities::profile::repo::{InsertProfileFn, QueryProfileByUserFn};
use crate::error::{Result, ServerSideError};
use crate::schemas::profile::ProfileCreateMultipart;
use crate::{
    api_response::ApiResponse, app_state::AppState,
    common::entities::profile::repo::QueryProfileFn, schemas::profile::ProfileResponder,
};
use actix_multipart::form::MultipartForm;
use actix_web::web;
use serde_json::{json, Value};
use std::fmt::Debug;
use std::io::Read;
use tracing::{info, instrument};

#[instrument(skip(app_data, profile))]
pub(crate) async fn create_profile<T: Debug + InsertProfileFn>(
    app_data: web::Data<AppState<T>>,
    profile: MultipartForm<ProfileCreateMultipart>,
) -> Result<ApiResponse<Value>> {
    info!("Create profile handler called");

    let result = app_data
        .db_repo
        .insert_profile(profile.into_inner().try_into()?)
        .await?;

    Ok(ApiResponse::created(json!({
        "message": "Profile created successfully",
        "profile_id": result
    })))
}

#[instrument(skip(app_data))]
pub(crate) async fn get_profile<T: Debug + QueryProfileFn>(
    app_data: web::Data<AppState<T>>,
    path: web::Path<i64>,
) -> Result<ApiResponse<ProfileResponder>> {
    info!("Get profile handler called for id: {}", path);
    let profile_id = path.into_inner();

    let profile = app_data.db_repo.query_profile(profile_id).await?;
    match profile {
        Some(profile) => Ok(ApiResponse::ok(profile.into())),
        None => Err(ServerSideError::ProfileNotFound(format!(
            "No profile found with id: {profile_id}"
        ))
        .into()),
    }
}

pub(crate) async fn get_profile_by_user_name<T: Debug + QueryProfileByUserFn>(
    app_data: web::Data<AppState<T>>,
    path: web::Path<String>,
) -> Result<ApiResponse<ProfileResponder>> {
    info!(
        "Get profile by user name handler called for user_name: {}",
        path
    );
    let username = path.into_inner();
    let profile = app_data
        .db_repo
        .query_profile_by_user(username.clone())
        .await?;

    let profile = profile
        .map(ProfileResponder::from)
        .ok_or(ServerSideError::ProfileNotFound(format!(
            "No profile found with user_name: {username}"
        )))?;

    Ok(ApiResponse::ok(profile))
}

impl From<ProfileQueryResult> for ProfileResponder {
    fn from(item: ProfileQueryResult) -> Self {
        ProfileResponder {
            id: item.id,
            created_at: item.created_at,
            user_name: item.user_name,
            full_name: item.full_name,
            description: item.description,
            region: item.region,
            main_url: item.main_url,
            avatar: item.avatar,
        }
    }
}

impl TryFrom<ProfileCreateMultipart> for ProfileCreate {
    type Error = ServerSideError;
    fn try_from(value: ProfileCreateMultipart) -> std::result::Result<Self, Self::Error> {
        let profile = ProfileCreate {
            user_name: value.user_name.to_string(),
            full_name: value.full_name.to_string(),
            description: value.description.to_string(),
            region: match value.region {
                Some(region) => Some(region.to_string()),
                None => None,
            },
            main_url: match value.main_url {
                Some(url) => Some(url.to_string()),
                None => None,
            },
            avatar: match value.avatar {
                Some(avatar) => {
                    let mut buffer: Vec<u8> = Vec::new();
                    avatar
                        .file
                        .as_file()
                        .read_to_end(buffer.as_mut())
                        .map_err(|err| {
                            ServerSideError::FileReadError(format!(
                                "Failed to read avatar file: {}",
                                err
                            ))
                        })?;
                    Some(buffer)
                },
                None => None,
            },
        };
        Ok(profile)
    }
}
