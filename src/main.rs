use __template__::{
    app,
    common::context::{init_redis, REDIS},
};
use axum::serve;
use dotenv::dotenv;
use std::{env, error::Error, io::stdout, net::SocketAddr};
use tokio::net::TcpListener;
use tracing::metadata::LevelFilter;
use tracing_appender::rolling;
use tracing_subscriber::{fmt, prelude::__tracing_subscriber_SubscriberExt, Layer};

fn init_tracing() -> Result<(), Box<dyn Error>> {
    let file_appender = rolling::daily("logs", "cas_server.log");
    let (writer, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = tracing_subscriber::registry().with(
        fmt::Layer::new()
            .with_writer(writer)
            .with_ansi(false)
            .with_filter(LevelFilter::INFO),
    );
    let mode = env::var("MODE")?;
    if mode == "DEBUG" {
        let subscriber = subscriber.with(
            fmt::Layer::new()
                .with_writer(stdout)
                .with_filter(LevelFilter::DEBUG),
        );
        tracing::subscriber::set_global_default(subscriber)?;
    } else {
        tracing::subscriber::set_global_default(subscriber)?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    REDIS.get_or_init(init_redis).await;

    init_tracing()?;

    let host = env::var("HOST")?;
    let port = env::var("PORT")?;
    let address: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::debug!("cas server is listening on {}", address);

    let app = app::init();
    let listener = TcpListener::bind(address).await?;
    serve(listener, app).await?;
    Ok(())
}
