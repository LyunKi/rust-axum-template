use __template__::app;
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use dotenv::dotenv;
use http_body_util::BodyExt;
use tower::{Service, ServiceExt};

#[tokio::test]
async fn test_health_check() {
    dotenv().ok();
    let app = app::init().await;
    let request = Request::builder()
        .uri("/health-check")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"Hello, world!");
}

#[tokio::test]
async fn test_i18n() {
    dotenv().ok();
    let mut app = app::init().await.into_service();
    let request = Request::builder()
        .uri("/demo/i18n?name=world")
        .body(Body::empty())
        .unwrap();
    let response: http::Response<Body> = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    // default to use en language
    assert_eq!(&body[..], b"Hello, world!");

    let uri = form_urlencoded::Serializer::new(String::new())
        .append_pair("name", "世界")
        .finish();
    let request: Request<Body> = Request::builder()
        .header("Accept-Language", "zh")
        .uri(format!("/demo/i18n?{uri}"))
        .body(Body::empty())
        .unwrap();
    let response: http::Response<Body> = ServiceExt::<Request<Body>>::ready(&mut app)
        .await
        .unwrap()
        .call(request)
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(String::from_utf8_lossy(&body[..]), "你好，世界！");
}
