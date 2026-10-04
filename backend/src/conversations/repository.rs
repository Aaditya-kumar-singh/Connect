use crate::{
    conversations::models::{ConversationDetail, DirectConversation},
    domain::errors::AppError,
};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

fn pair_key(a: Uuid, b: Uuid) -> String {
    let (low, high) = if a < b { (a, b) } else { (b, a) };
    format!("{low}:{high}")
}

pub async fn create_direct(
    pool: &PgPool,
    user_id: Uuid,
    other_user_id: Uuid,
) -> Result<Uuid, AppError> {
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(pair_key(user_id, other_user_id))
        .execute(&mut *tx)
        .await?;

    if let Some(id) = sqlx::query_scalar::<_, Uuid>(
        "SELECT c.id FROM conversations c
         JOIN conversation_members a ON a.conversation_id=c.id AND a.user_id=$1 AND a.left_at IS NULL
         JOIN conversation_members b ON b.conversation_id=c.id AND b.user_id=$2 AND b.left_at IS NULL
         WHERE c.conversation_type='DIRECT' LIMIT 1",
    )
    .bind(user_id).bind(other_user_id).fetch_optional(&mut *tx).await? {
        tx.commit().await?;
        return Ok(id);
    }

    let id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO conversations (conversation_type) VALUES ('DIRECT') RETURNING id",
    )
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO conversation_members (conversation_id,user_id) VALUES ($1,$2),($1,$3)",
    )
    .bind(id)
    .bind(user_id)
    .bind(other_user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(id)
}

pub async fn get_direct(
    pool: &PgPool,
    user_id: Uuid,
    conversation_id: Uuid,
) -> Result<ConversationDetail, AppError> {
    let conversation = sqlx::query_as::<_, DirectConversation>(
        "SELECT c.id,c.conversation_type,c.created_at,c.updated_at,
                u.id AS other_user_id,u.display_name AS other_display_name,p.avatar_url AS other_avatar_url
         FROM conversations c
         JOIN conversation_members me ON me.conversation_id=c.id AND me.user_id=$1 AND me.left_at IS NULL
         JOIN conversation_members other ON other.conversation_id=c.id AND other.user_id<>$1 AND other.left_at IS NULL
         JOIN users u ON u.id=other.user_id
         LEFT JOIN user_profiles p ON p.user_id=u.id
         WHERE c.id=$2 AND c.conversation_type='DIRECT' AND u.status='ACTIVE'",
    ).bind(user_id).bind(conversation_id).fetch_optional(pool).await?
     .ok_or_else(|| AppError::NotFound("Conversation not found".into()))?;

    let member_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM conversation_members WHERE conversation_id=$1 AND left_at IS NULL ORDER BY user_id",
    ).bind(conversation_id).fetch_all(pool).await?;

    Ok(ConversationDetail {
        conversation,
        member_ids,
    })
}

pub async fn list_direct(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
    cursor: Option<(DateTime<Utc>, Uuid)>,
) -> Result<Vec<DirectConversation>, AppError> {
    let rows = if let Some((timestamp, id)) = cursor {
        sqlx::query_as::<_, DirectConversation>(
            "SELECT c.id,c.conversation_type,c.created_at,c.updated_at,
                    u.id AS other_user_id,u.display_name AS other_display_name,p.avatar_url AS other_avatar_url
             FROM conversations c
             JOIN conversation_members me ON me.conversation_id=c.id AND me.user_id=$1 AND me.left_at IS NULL
             JOIN conversation_members other ON other.conversation_id=c.id AND other.user_id<>$1 AND other.left_at IS NULL
             JOIN users u ON u.id=other.user_id
             LEFT JOIN user_profiles p ON p.user_id=u.id
             WHERE c.conversation_type='DIRECT' AND u.status='ACTIVE'
               AND (c.updated_at,c.id)<($2,$3)
             ORDER BY c.updated_at DESC,c.id DESC LIMIT $4",
        ).bind(user_id).bind(timestamp).bind(id).bind(limit).fetch_all(pool).await?
    } else {
        sqlx::query_as::<_, DirectConversation>(
            "SELECT c.id,c.conversation_type,c.created_at,c.updated_at,
                    u.id AS other_user_id,u.display_name AS other_display_name,p.avatar_url AS other_avatar_url
             FROM conversations c
             JOIN conversation_members me ON me.conversation_id=c.id AND me.user_id=$1 AND me.left_at IS NULL
             JOIN conversation_members other ON other.conversation_id=c.id AND other.user_id<>$1 AND other.left_at IS NULL
             JOIN users u ON u.id=other.user_id
             LEFT JOIN user_profiles p ON p.user_id=u.id
             WHERE c.conversation_type='DIRECT' AND u.status='ACTIVE'
             ORDER BY c.updated_at DESC,c.id DESC LIMIT $2",
        ).bind(user_id).bind(limit).fetch_all(pool).await?
    };
    Ok(rows)
}

pub async fn members_active(
    pool: &PgPool,
    user_id: Uuid,
    other_user_id: Uuid,
) -> Result<bool, AppError> {
    sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
            SELECT 1 FROM conversation_members m
            WHERE m.user_id=$1 AND m.left_at IS NULL
              AND EXISTS (
                SELECT 1 FROM conversation_members x
                WHERE x.conversation_id=m.conversation_id AND x.user_id=$2 AND x.left_at IS NULL
              )
        )",
    )
    .bind(user_id)
    .bind(other_user_id)
    .fetch_one(pool)
    .await
    .map_err(AppError::Database)
}
