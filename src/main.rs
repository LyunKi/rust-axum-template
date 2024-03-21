use axum::{response::Html, routing::get, Router};
use std::env;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let host = env::var("HOST")?;
    let port = env::var("PORT")?;
    let address: std::net::SocketAddr = format!("{}:{}", host, port).parse()?;

    let app = Router::new().route("/", get(handler));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
