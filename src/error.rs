use axum::{ http::StatusCode, response::Response};
use thiserror::Error;
use immortal_axum_utils::error::ErrorResponse;


#[derive(Debug, Error)]
pub enum ServerError {
    #[error("{1}")]
    BusinessError(StatusCode, &'static str),
}

pub static INVALID_PARAMS: &str = "error.invalid_params";
pub static USER_NOT_FOUND: &str = "error.business.user_not_found";

impl axum::response::IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::BusinessError(status, code) => {
                ErrorResponse::new(code.to_string(), status).into_response()
            }
        }
    }
}
