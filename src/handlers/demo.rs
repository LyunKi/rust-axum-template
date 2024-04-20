use std::collections::HashMap;

use axum::{extract::Query, http::StatusCode};
use axum_extra::TypedHeader;
use immortal_axum_utils::{error::ErrorResponse, extractors::headers::AcceptLanguage};
use immortal_intl_rs::t;
use serde::Deserialize;

use crate::error::INVALID_PARAMS;

#[derive(Deserialize)]
pub struct I18nParams {
    name: Option<String>,
}

pub async fn i18n_demo(
    Query(params): Query<I18nParams>,
    TypedHeader(accept_language): TypedHeader<AcceptLanguage>,
) -> Result<String, ErrorResponse> {
    let args = HashMap::from([(
        "name".to_owned(),
        params.name.ok_or(ErrorResponse::new(INVALID_PARAMS, StatusCode::BAD_REQUEST))?
    )]);
    let hello = t!("demo.hello", accept_language:accept_language, args:args);
    Ok(hello)
}
