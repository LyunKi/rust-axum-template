use immortal_axum_macro::IntoResponse;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct CreateUserReq {
    #[validate(length(min = 1, max = 10, code = "error.business.name_limit"))]
    pub name: String,
}

#[derive(Serialize, IntoResponse, Deserialize)]
pub struct UserRspDto {
    pub id: Uuid,
    pub name: String,
}

#[derive(Validate, Serialize, Deserialize)]
pub struct UpdateUserReq {
    #[validate(length(min = 1, max = 10, code = "error.business.name_limit"))]
    pub name: String,
}

#[derive(Serialize, IntoResponse)]
pub struct DeleteUserRspDto {
    pub affected_rows: usize,
}
