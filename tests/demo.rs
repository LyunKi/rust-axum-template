use __template__::{
    app,
    dto::{CreateUserReq, Paginated, SetRedisValueReq, UpdateUserReq, UserRspDto},
};
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use dotenv::dotenv;
use http_body_util::BodyExt;
use immortal_axum_utils::{error::ErrorResponseBody, test_helpers::TestClient};
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

#[tokio::test]
async fn test_validation() {
    dotenv().ok();
    let app: axum::Router = app::init().await;
    let create_user_req = CreateUserReq {
        name: "".to_string(),
    };
    let client = TestClient::new(app);
    let response = client.post("/demo/users").json(&create_user_req).await;

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);

    let body = response.json::<ErrorResponseBody>().await;
    assert_eq!(&body.code, "error.bad_request");
    assert_eq!(
        body.children.unwrap().first().unwrap().code,
        "error.business.name_limit"
    );
}

#[tokio::test]
async fn test_crud() {
    dotenv().ok();
    let app = app::init().await;
    let create_user_req = CreateUserReq {
        name: "1".to_string(),
    };
    let client = TestClient::new(app);
    // create
    let response = client.post("/demo/users").json(&create_user_req).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json::<UserRspDto>().await;
    let id = body.id;
    assert_eq!(body.name, "1");
    // get one
    let response = client.get(&format!("/demo/users/{id}")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json::<UserRspDto>().await;
    assert_eq!(body.name, "1");
    // update
    let req = UpdateUserReq {
        name: "2".to_string(),
    };
    let response = client.put(&format!("/demo/users/{id}")).json(&req).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json::<UserRspDto>().await;
    assert_eq!(body.name, "2");
    // get all
    let response = client.get("/demo/users?page=1&page_size=10").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json::<Paginated<UserRspDto>>().await;
    let before_total_num = body.total_num;
    // delete
    let response = client.delete(&format!("/demo/users/{id}")).await;
    assert_eq!(response.status(), StatusCode::OK);
    // get all
    let response = client.get("/demo/users?page=1&page_size=10").await;
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.json::<Paginated<UserRspDto>>().await;
    let current_total_num = body.total_num;
    assert_eq!(current_total_num, before_total_num - 1);
}

#[tokio::test]
async fn test_redis() {
    dotenv().ok();
    let app = app::init().await;
    let client = TestClient::new(app);
    let id = "redis_key";
    // set
    let req = SetRedisValueReq {
        value: "redis_value".to_string(),
    };
    let response = client.put(&format!("/demo/redis/{id}")).json(&req).await;
    assert_eq!(response.status(), StatusCode::OK);

    // get
    let response = client.get(&format!("/demo/redis/{id}")).await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.text().await, "redis_value");

    // delete
    let response = client.delete(&format!("/demo/redis/{id}")).await;
    assert_eq!(response.status(), StatusCode::OK);
}
