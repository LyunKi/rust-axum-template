use axum::{
    body::Body,
    extract::connect_info::MockConnectInfo,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tower::{Service, ServiceExt};

#[tokio::test]
async fn hello_world() {
    let app = app();

    let response = app
        .oneshot(Request::builder().uri("/demo/i18n").unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"Hello, World!");
}
