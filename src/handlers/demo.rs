use immortal_axum_utils::{error::ErrorResponse, extractors::headers::AcceptLanguage};
use axum_extra::TypedHeader;

pub async fn i18n_demo(TypedHeader(acceptLanguage): TypedHeader<AcceptLanguage>) -> Result<(), ErrorResponse> {
    Ok(())
}
