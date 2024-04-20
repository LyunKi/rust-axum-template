use immortal_axum_macro::IntoResponse;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Validate, Serialize, Deserialize)]
pub struct CreateUserReq {
    #[validate(length(min = 1, max = 10, code="error.business.name_limit"))]
    pub name: String,
}

#[derive(Serialize, IntoResponse)]
pub struct CreateUserRsp {
    pub id: Uuid,
}
