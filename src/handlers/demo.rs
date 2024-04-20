use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    http::StatusCode, 
};
use axum_extra::TypedHeader;
use entity::user;
use immortal_axum_utils::{
    error::ErrorResponse,
    extractors::{headers::AcceptLanguage, validation::ValidatedJson},
};
use immortal_intl_rs::t;
use sea_orm::{Set, TransactionTrait, ActiveModelTrait};
use serde::Deserialize;
use uuid::Uuid;
use crate::{
    common::context::AppState,
    dto::{CreateUserReq, CreateUserRsp},
    error::INVALID_PARAMS,
};

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
        params
            .name
            .ok_or(ErrorResponse::new(INVALID_PARAMS, StatusCode::BAD_REQUEST))?,
    )]);
    let hello = t!("demo.hello", accept_language:accept_language, args:args);
    Ok(hello)
}

pub async fn create_user(
    State(state): State<AppState>,
    ValidatedJson(user): ValidatedJson<CreateUserReq>,
) -> Result<CreateUserRsp, ErrorResponse> {
    let db = state.db;
    let transaction = db.begin().await?;
    // just a demo, no need to validate
    let result = user::ActiveModel {
        name: Set(user.name),
        id: Set(Uuid::new_v4())
    }
    .insert(&transaction)
    .await
    .map(|created| CreateUserRsp { id: created.id })?;
    transaction.commit().await?;
    Ok(result)
}

pub async fn get_user() {}

pub async fn update_user() {}

pub async fn delete_user() {}

pub async fn get_user_list() {}
