use actix_web::{
    body,
    http::{self, header::ContentType},
    HttpResponse, ResponseError,
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum ServerSideError {
    #[error("Unknown Internal Server Error")]
    InternalServerError,
}

#[derive(Debug, Serialize, thiserror::Error)]
#[serde()]
pub enum ClientSideError {
    #[error("Internal Server Error")]
    InternalServerError,
}

impl From<ServerSideError> for ClientSideError {
    fn from(value: ServerSideError) -> Self {
        println!("Error: {}", value);
        match value {
            ServerSideError::InternalServerError => ClientSideError::InternalServerError,
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
