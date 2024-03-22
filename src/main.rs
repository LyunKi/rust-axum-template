use axum::{routing::get, serve, Router};
use dotenv::dotenv;
use std::{env, error::Error, net::SocketAddr};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let host = env::var("HOST")?;
    let port = env::var("PORT")?;
    let address: SocketAddr = format!("{}:{}", host, port).parse()?;

    let app = Router::new().route("/health-check", get(handler));
    let listener = TcpListener::bind(address).await?;
    serve(listener, app).await?;
    Ok(())
}

async fn handler() -> String {
    String::from("Hello, world")
}
