use std::{convert::Infallible, fmt::Debug};
use axum::{
    http::{HeaderValue,header},
    response::Response,
    body,
        extract::Request,
};
use futures_util::future::BoxFuture;
use immortal_intl_rs::TranslationConfig;
use tower::{Layer, Service};
use immortal_axum_utils::error::{ServerError, TmpError};
use axum::response::IntoResponse;

#[derive(Clone, Debug)]
pub struct ErrorTranslator<S> {
    inner: S,
}

#[derive(Clone, Debug)]
pub struct ErrorTranslatorLayer {}

impl ErrorTranslatorLayer {
    pub fn new() -> Self {
        ErrorTranslatorLayer {}
    }
}

impl<S> Layer<S> for ErrorTranslatorLayer {
    type Service = ErrorTranslator<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ErrorTranslator { inner }
    }
}

impl<ReqBody, S> Service<Request<ReqBody>> for ErrorTranslator<S>
where
    S: Service<Request<ReqBody>, Response = Response, Error = Infallible> + Send,
    S::Future: Send + 'static,
    S::Error: Debug,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let accept_language: Option<String> = req
            .headers()
            .get(header::ACCEPT_LANGUAGE)
            .map(HeaderValue::to_str)
            .and_then(|result| result.ok())
            .map(ToString::to_string);
        let res_future = self.inner.call(req);
        Box::pin(async move {
            let result = res_future.await;
            match result {
                Ok(response) if !response.status().is_success() => {
                    let status = response.status();
                    let body = response.into_body();
                    let mut tmp_error = body::to_bytes(body,usize::MAX)
                        .await
                        .ok()
                        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
                        .unwrap_or(TmpError::from(ServerError::InternalServerError));
                    let translation_config = TranslationConfig {
                        accept_language,
                        args: tmp_error.args.take(),
                        ..Default::default()
                    };
                    let error_body = tmp_error.translate(&translation_config);
                    Ok((status, error_body).into_response())
                }
                Ok(response) => Ok(response),
                Err(err) => Err(err),
            }
        })
    }
}
