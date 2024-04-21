use std::collections::HashMap;

use crate::{
    common::context::AppState,
    dto::{CreateUserReq, DeleteUserRspDto, UpdateUserReq, UserRspDto},
    error::{INVALID_PARAMS, USER_NOT_FOUND},
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use axum_extra::TypedHeader;
use entity::user;
use immortal_axum_utils::{
    error::ErrorResponse,
    extractors::{headers::AcceptLanguage, validation::ValidatedJson},
};
use immortal_intl_rs::t;
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, ModelTrait, Set, TransactionTrait,
};
use serde::Deserialize;
use uuid::Uuid;

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
    State(AppState { db, redis: _ }): State<AppState>,
    ValidatedJson(user): ValidatedJson<CreateUserReq>,
) -> Result<UserRspDto, ErrorResponse> {
    let transaction = db.begin().await?;
    // just a demo, no need to validate
    let result = user::ActiveModel {
        name: Set(user.name),
        id: Set(Uuid::new_v4()),
    }
    .insert(&transaction)
    .await
    .map(|created| UserRspDto {
        id: created.id,
        name: created.name,
    })?;
    transaction.commit().await?;
    Ok(result)
}

async fn get_user_by_id(db: &DatabaseConnection, id: Uuid) -> Option<user::Model> {
    user::Entity::find_by_id(id).one(db).await.ok().flatten()
}

pub async fn get_user(
    State(AppState { db, redis: _ }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<UserRspDto, ErrorResponse> {
    let user = get_user_by_id(&db, id)
        .await
        .map(|model| UserRspDto {
            id: model.id,
            name: model.name,
        })
        .ok_or(ErrorResponse::new(USER_NOT_FOUND, StatusCode::NOT_FOUND))?;
    Ok(user)
}

pub async fn update_user(
    State(AppState { db, redis: _ }): State<AppState>,
    Path(id): Path<Uuid>,
    ValidatedJson(update_user): ValidatedJson<UpdateUserReq>,
) -> Result<UserRspDto, ErrorResponse> {
    let user = get_user_by_id(&db, id)
        .await
        .ok_or(ErrorResponse::new(USER_NOT_FOUND, StatusCode::NOT_FOUND))?;
    let mut user_active_model: user::ActiveModel = user.into();
    user_active_model.name = Set(update_user.name.clone());
    user_active_model.update(&db).await?;
    Ok(UserRspDto {
        id,
        name: update_user.name,
    })
}

pub async fn delete_user(
    State(AppState { db, redis: _ }): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<DeleteUserRspDto, ErrorResponse> {
    let target = get_user_by_id(&db, id).await;
    if target.is_none() {
        Ok(DeleteUserRspDto { affected_rows: 0 })
    } else {
        let result = target.unwrap().delete(&db).await?;
        Ok(DeleteUserRspDto {
            affected_rows: result.rows_affected as usize,
        })
    }
}

pub async fn get_user_list(
    State(AppState { db, redis: _ }): State<AppState>,
) -> Result<Json<Vec<UserRspDto>>, ErrorResponse> {
    Ok(user::Entity::find().all(&db).await.map(|users| {
        Json(
            users
                .into_iter()
                .map(|model| UserRspDto {
                    id: model.id,
                    name: model.name,
                })
                .collect(),
        )
    })?)
}
