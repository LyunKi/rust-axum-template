use axum::{extract::rejection::JsonRejection, http::StatusCode, response::Response};
use immortal_intl_rs::{t, TranslationConfig};
use immortal_axum_macro::IntoResponse;
use jsonwebtoken::errors::Error as JwtError;
use redis::RedisError;
use sea_orm::DbErr;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};
use thiserror::Error;
use uuid::Error as UuidError;
use validator::{ValidationErrors, ValidationErrorsKind};

impl From<RedisError> for ServerError {
    fn from(err: RedisError) -> Self {
        Self::internal_error(err)
    }
}

impl From<UuidError> for ServerError {
    fn from(err: UuidError) -> Self {
        Self::internal_error(err)
    }
}

impl From<JwtError> for ServerError {
    fn from(err: JwtError) -> Self {
        Self::internal_error(err)
    }
}

impl From<DbErr> for ServerError {
    fn from(err: DbErr) -> Self {
        Self::internal_error(err)
    }
}

const VALIDATION_ERROR: &'static str = "error.business.failed_params_validation";
pub const MOBILE_REGISTERED_ERROR: &'static str = "error.business.mobile_registered";
pub const REAUTH: &'static str = "error.business.reauth";
pub const INVALID_AUTH: &'static str = "error.business.invalid_auth";
pub const INVALID_VERIFICATION_CODE: &'static str = "error.business.invalid_verification_code";
pub const TARGET_USER_NOT_FOUND: &'static str = "error.business.target_user_not_found";
const INTERNAL_SERVER_ERROR: &'static str = "error.internal_error";
const TIMEOUT_ERROR: &'static str = "error.timeout_error";
const SERVICE_UNAVAILABLE: &'static str = "error.unavailable";
const BAD_REQUEST: &'static str = "error.bad_request";

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("{TIMEOUT_ERROR}")]
    TimeoutError,
    #[error("{INTERNAL_SERVER_ERROR}")]
    InternalServerError,
    #[error("{SERVICE_UNAVAILABLE}")]
    ServiceUnavailable,
    #[error("{BAD_REQUEST}")]
    InvalidBody,
    #[error("{BAD_REQUEST}")]
    BadRequest,
    #[error("{1}")]
    BusinessError(StatusCode, &'static str),
    #[error(transparent)]
    AxumJsonRejection(#[from] JsonRejection),
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
}

impl ServerError {
    pub fn internal_error<T: fmt::Debug>(err: T) -> Self {
        ServerError::decorate_error(err, Self::InternalServerError)
    }

    pub fn decorate_error<T: fmt::Debug>(err: T, show_err: ServerError) -> Self {
        tracing::warn!(
            "Error occured: {:#?} ,\n and would be shown as {:#?}",
            err,
            show_err
        );
        show_err
    }
}

impl From<ServerError> for TmpError {
    fn from(error: ServerError) -> Self {
        match error {
            _ => TmpError::from_code(error.to_string()),
        }
    }
}

impl axum::response::IntoResponse for ServerError {
    fn into_response(self) -> Response {
        match self {
            ServerError::TimeoutError => {
                (StatusCode::REQUEST_TIMEOUT, TmpError::from(self)).into_response()
            }
            ServerError::ServiceUnavailable => {
                (StatusCode::SERVICE_UNAVAILABLE, TmpError::from(self)).into_response()
            }
            ServerError::InvalidBody => {
                (StatusCode::BAD_REQUEST, TmpError::from(self)).into_response()
            }
            ServerError::AxumJsonRejection(err) => (
                StatusCode::BAD_REQUEST,
                TmpError::from(ServerError::decorate_error(err, ServerError::BadRequest)),
            )
                .into_response(),
            ServerError::BusinessError(status, code) => {
                (status, TmpError::from_code(code.to_string())).into_response()
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, TmpError::from(self)).into_response(),
        }
    }
}
