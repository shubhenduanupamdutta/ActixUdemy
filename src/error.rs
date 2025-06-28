use actix_web::{
    body,
    http::{self, header::ContentType},
    HttpResponse, ResponseError,
};
use serde::Serialize;
use serde_json::json;
use tracing::error;

#[derive(Debug, thiserror::Error)]
pub enum ServerSideError {
    #[error("Unknown Internal Server Error: {0}")]
    InternalServerError(String),
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde()]
pub enum ClientSideError {
    #[error("Internal Server Error")]
    InternalServerError,
}

impl From<ServerSideError> for ClientSideError {
    fn from(value: ServerSideError) -> Self {
        let error_id = uuid::Uuid::new_v4();
        error!(error_id = %error_id, error = %value);

        match value {
            ServerSideError::InternalServerError(_) => ClientSideError::InternalServerError,
        }
    }
}

impl ResponseError for ClientSideError {
    fn status_code(&self) -> http::StatusCode {
        match self {
            ClientSideError::InternalServerError => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse<body::BoxBody> {
        let json_body = json!({"error": self.to_string()});
        HttpResponse::build(self.status_code())
            .content_type(ContentType::json())
            .json(json_body)
    }
}

pub type Result<T> = std::result::Result<T, ClientSideError>;
