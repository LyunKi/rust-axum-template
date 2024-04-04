use async_trait::async_trait;
use axum::{
    body::HttpBody,
    extract::{FromRequest, RequestParts},
    BoxError, Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::error::ServerError;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    B: HttpBody + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = ServerError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}
