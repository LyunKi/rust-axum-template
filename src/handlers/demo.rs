use crate::{error::ServerError, extractors::AcceptLanguage};
use axum_extra::{headers::{self, AcceptRanges}, TypedHeader};

pub async fn i18n_demo(TypedHeader(user_agent): TypedHeader<AcceptRanges>) -> Result<(), ServerError> {
    Ok(())
}
