use redis::aio::MultiplexedConnection;
use sea_orm::{Database, DatabaseConnection};
use std::{env, sync::Arc};
use tokio::sync::Mutex;
use tokio::sync::OnceCell;

pub type RedisConnection = Arc<Mutex<MultiplexedConnection>>;

pub type Redis = OnceCell<RedisConnection>;

#[derive(Clone)]
pub struct AppState {
    db: DatabaseConnection,
    redis: MultiplexedConnection,
}

pub static APP_STATE: OnceCell<AppState> = OnceCell::const_new();

pub async fn init_app_state() -> AppState {
    let redis_pass: String = env::var("REDIS_PASS").expect("REDIS_PASS must be set");
    let client = redis::Client::open(format!("redis://:{}@127.0.0.1/", redis_pass))
        .expect("Redis server had been started");
    let redis: MultiplexedConnection = client.get_multiplexed_async_connection().await.unwrap();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(db_url).await.unwrap();

    AppState { db, redis }
}

