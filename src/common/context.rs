use std::{env, sync::Arc};

use redis::aio::MultiplexedConnection;
use tokio::sync::Mutex;
use tokio::sync::OnceCell;

pub type RedisConnection = Arc<Mutex<MultiplexedConnection>>;

pub type Redis = OnceCell<RedisConnection>;

pub static REDIS: Redis = OnceCell::const_new();

pub async fn init_redis() -> RedisConnection {
    let redis_pass = env::var("REDIS_PASS").expect("REDIS_PASS must be set");
    let client = redis::Client::open(format!("redis://:{}@127.0.0.1/", redis_pass))
        .expect("Redis server had been started");
    let redis: MultiplexedConnection = client.get_multiplexed_async_connection ().await.unwrap();
    Arc::new(Mutex::new(redis))
}
