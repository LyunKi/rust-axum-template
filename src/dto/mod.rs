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

#[derive(Deserialize)]
pub struct PaginationParams {
    pub page_size: Option<u64>,
    pub page: u64,
}

#[derive(Serialize, IntoResponse, Deserialize)]
pub struct Paginated<T: Serialize> {
    pub page: u64,
    pub page_size: u64,
    pub total_num: u64,
    pub total_page: u64,
    pub items: Vec<T>,
}


#[derive(Validate, Serialize, Deserialize)]
pub struct SetRedisValueReq {
    pub value: String,
}

