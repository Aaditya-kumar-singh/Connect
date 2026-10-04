use redis::Client;
use std::time::Duration;

pub fn create_client(redis_url: &str) -> Result<Client, redis::RedisError> {
    Client::open(redis_url)
}

pub async fn check_health(client: &Client) -> Result<Duration, redis::RedisError> {
    let start = std::time::Instant::now();
    let mut conn = client.get_multiplexed_async_connection().await?;
    let _: () = redis::cmd("PING").query_async(&mut conn).await?;
    Ok(start.elapsed())
}
