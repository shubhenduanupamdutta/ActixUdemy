use crate::error::{IntoClientResult as _, ServerSideError};
use actix_web::{
    body::BoxBody,
    http::{header::ContentType, StatusCode},
    HttpResponse, Responder, ResponseError,
};
use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct ApiResponse<T: Serialize> {
    #[serde(skip_serializing)]
    status_code: StatusCode,
    data: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub(crate) fn new(status_code: StatusCode, data: T) -> Self {
        ApiResponse { status_code, data }
    }

    pub(crate) fn ok(data: T) -> Self {
        ApiResponse { status_code: StatusCode::OK, data }
    }

    pub(crate) fn created(data: T) -> Self {
        ApiResponse { status_code: StatusCode::CREATED, data }
    }
}

#[allow(unused_variables)]
impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = BoxBody;

    fn respond_to(self, req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        let output = serde_json::to_value(&self.data)
            .map_err(ServerSideError::from)
            .into_client_result();

        match output {
            Ok(value) => HttpResponse::build(self.status_code)
                .content_type(ContentType::json())
                .json(value),
            Err(error) => error.error_response(),
        }
    }
}
