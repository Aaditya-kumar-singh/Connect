use redis::aio::ConnectionManager;
use redis::Client;

pub async fn create_pool(redis_url: &str) -> Result<ConnectionManager, redis::RedisError> {
    let client = Client::open(redis_url)?;
    ConnectionManager::new(client).await
}
