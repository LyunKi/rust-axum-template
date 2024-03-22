use axum::{
    body::{Bytes,Body,self},http::Method, error_handling::HandleErrorLayer, http::Request, response::IntoResponse,
    response::Response, routing, Extension, BoxError, Router, middleware::{Next, self}
};
use sea_orm::DatabaseConnection;
use std::{
    env,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tower_http::ServiceBuilderExt;
use tower_http::{
    compression::CompressionLayer,
    request_id::{MakeRequestId, RequestId},
    trace::TraceLayer,
};
use tracing::debug_span;
use crate::{
    common::constants::{REQUEST, RESPONSE},
    error::ServerError,
    middlewares::ErrorTranslatorLayer,
    utils::redis::REDIS,
};

#[derive(Clone, Default)]
struct RequestIdGenerator {
    counter: Arc<AtomicU64>,
}

impl MakeRequestId for RequestIdGenerator {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id = self
            .counter
            .fetch_add(1, Ordering::SeqCst)
            .to_string()
            .parse()
            .unwrap();

        Some(RequestId::new(request_id))
    }
}

async fn buffer_and_print<B>(direction: &'static str, body: B) -> Result<Bytes, ServerError>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Debug,
{
    let bytes = match body::to_bytes(body).await {
        Ok(bytes) => bytes,
        Err(err) => {
            return Err(ServerError::decorate_error(err, ServerError::InvalidBody));
        }
    };
    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::info!("{} body = {:?}", direction, body);
    }
    Ok(bytes)
}

async fn print_request_response(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<impl IntoResponse, ServerError> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print(REQUEST, body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print(RESPONSE, body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));
    Ok(res)
}

pub fn init(db: &DatabaseConnection) -> Router {
    Router::new()
        .route("/health=check", routing::get(|| async { "Hello, World!" }))
        .layer(
            ServiceBuilder::new()
                .layer(CompressionLayer::new())
                .set_x_request_id(RequestIdGenerator::default())
                .layer(
                    TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                        let request_id = request.extensions().get::<RequestId>().unwrap();
                        let request_span = format!(
                            "request id {:?} method={} uri={} version={:?}",
                            request_id,
                            request.method(),
                            request.uri(),
                            request.version(),
                        );
                        debug_span!("request", message = %request_span)
                    }),
                )
                // propagate `x-request-id` headers from request to response
                .propagate_x_request_id()
                .layer(ErrorTranslatorLayer::new())
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    tracing::error!("Unhandled Error occureed, origin error is {:#?}", error);
                    if error.is::<tower::timeout::error::Elapsed>() {
                        ServerError::TimeoutError
                    } else if error.is::<tower::load_shed::error::Overloaded>() {
                        ServerError::ServiceUnavailable
                    } else {
                        ServerError::InternalServerError
                    }
                }))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10))
                .layer(middleware::from_fn(print_request_response))
                .layer(Extension(db.clone()))
                .layer(Extension(REDIS.get().unwrap().clone()))
                .into_inner(),
        )
        .layer(
            CorsLayer::new()
                .allow_credentials(true)
                .allow_headers(Any)
                .expose_headers(Any)
                .max_age(Duration::from_secs(60 * 60))
                .allow_origin(Origin::list(
                    env::var("ALLOW_ORIGINS")
                        .unwrap()
                        .split(",")
                        .map(|s| s.parse().unwrap()),
                ))
                .allow_methods(vec![
                    Method::OPTIONS,
                    Method::POST,
                    Method::GET,
                    Method::DELETE,
                    Method::PUT,
                ]),
        )
}
