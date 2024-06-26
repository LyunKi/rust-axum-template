use crate::{
    common::{
        constants::{REQUEST, RESPONSE},
        context::{init_app_state, APP_STATE},
    },
    handlers,
};
use anyhow::anyhow;
use axum::{
    body::{Body, Bytes},
    error_handling::HandleErrorLayer,
    http::Method,
    http::Request,
    middleware::{self, Next},
    response::IntoResponse,
    response::Response,
    routing, BoxError, Router,
};
use http_body_util::BodyExt;
use immortal_axum_utils::{
    error::ErrorResponse,
    middlewares::ErrorTranslatorLayer,
};
use std::{
    env,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tower::ServiceBuilder;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tower_http::ServiceBuilderExt;
use tower_http::{
    compression::CompressionLayer,
    request_id::{MakeRequestId, RequestId},
    trace::TraceLayer,
};
use tracing::debug_span;

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

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, ErrorResponse>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::error::Error,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err(anyhow!("failed to collect body: {}", err).into());
        }
    };
    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::info!("{} body = {:?}", direction, body);
    }
    Ok(bytes)
}

async fn print_request_response(
    req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, ErrorResponse> {
    let (parts, body) = req.into_parts();

    tracing::info!(
        "Tracing request, method = {:?} uri = {:?}",
        &parts.method,
        &parts.uri
    );

    let bytes = buffer_and_print(REQUEST, body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print(RESPONSE, body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));
    Ok(res)
}

pub async fn init() -> Router {
    Router::new()
        .route("/health-check", routing::get(|| async { "Hello, world!" }))
        .route("/demo/i18n", routing::get(handlers::i18n_demo))
        .route(
            "/demo/users",
            routing::post(handlers::create_user).get(handlers::get_user_list),
        )
        .route(
            "/demo/users/:id",
            routing::get(handlers::get_user)
                .delete(handlers::delete_user)
                .put(handlers::update_user),
        )
        .route(
            "/demo/redis/:id",
            routing::get(handlers::test_redis_get)
                .delete(handlers::test_redis_delete)
                .put(handlers::test_redis_set),
        )
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
                    ErrorResponse::internal_error(error)
                }))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10))
                .layer(middleware::from_fn(print_request_response))
                .into_inner(),
        )
        .layer(
            CorsLayer::new()
                .allow_headers(Any)
                .expose_headers(Any)
                .allow_origin(AllowOrigin::list(
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
        .with_state(APP_STATE.get_or_init(init_app_state).await.clone())
}
