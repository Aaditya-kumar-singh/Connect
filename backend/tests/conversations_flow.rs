use uuid::Uuid;
use ybm_connect::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    config::Config,
    conversations::service,
    infrastructure::{database, redis_client},
};

fn test_state() -> Option<AppState> {
    let db_url = std::env::var("TEST_DATABASE_URL").ok()?;
    let redis_url = std::env::var("TEST_REDIS_URL").ok()?;
    let mut config = Config::from_env().ok()?;
    config.database_url = db_url.clone();
    config.redis_url = redis_url.clone();
    Some(AppState::new(
        config,
        database::create_pool(&db_url, 5, 1).ok()?,
        redis_client::create_client(&redis_url).ok()?,
    ))
}

async fn insert_user(state: &AppState, email: &str) -> Uuid {
    sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO users (email,password_hash,display_name,status)
         VALUES ($1,'test','Conversation Test','ACTIVE') RETURNING id",
    )
    .bind(email)
    .fetch_one(state.db())
    .await
    .unwrap()
}

fn auth(user_id: Uuid) -> AuthenticatedSession {
    AuthenticatedSession {
        user_id,
        session_id: Uuid::new_v4(),
        device_id: Uuid::new_v4(),
        access_token_hash: "test".into(),
    }
}

#[tokio::test]
#[ignore = "requires PostgreSQL and Redis configured through TEST_DATABASE_URL and TEST_REDIS_URL"]
async fn conversation_flow_covers_duplicate_membership_block_and_pagination() {
    let state = test_state().unwrap();
    sqlx::migrate!("./migrations")
        .run(state.db())
        .await
        .unwrap();

    let suffix = Uuid::new_v4();
    let a = insert_user(&state, &format!("conv-a-{suffix}@example.com")).await;
    let b = insert_user(&state, &format!("conv-b-{suffix}@example.com")).await;
    let c = insert_user(&state, &format!("conv-c-{suffix}@example.com")).await;

    let first = service::create(&state, &auth(a), b).await.unwrap();
    let duplicate = service::create(&state, &auth(a), b).await.unwrap();
    assert_eq!(first.conversation.id, duplicate.conversation.id);

    let listed = service::list(&state, &auth(a), Some(1), None)
        .await
        .unwrap();
    assert_eq!(listed.items.len(), 1);

    let denied = service::detail(&state, &auth(c), first.conversation.id).await;
    assert!(denied.is_err());

    sqlx::query("INSERT INTO blocked_users (blocker_id,blocked_id) VALUES ($1,$2)")
        .bind(a)
        .bind(c)
        .execute(state.db())
        .await
        .unwrap();
    assert!(service::create(&state, &auth(a), c).await.is_err());

    sqlx::query("DELETE FROM users WHERE id IN ($1,$2,$3)")
        .bind(a)
        .bind(b)
        .bind(c)
        .execute(state.db())
        .await
        .unwrap();
}
