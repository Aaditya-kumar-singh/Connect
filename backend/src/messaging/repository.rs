use crate::{domain::errors::AppError, messaging::models::Message};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub struct NewMessage<'a> {
    pub message_id: Uuid,
    pub client_message_id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: &'a str,
    pub content_type: &'a str,
    pub reply_to_message_id: Option<Uuid>,
    pub forwarded_from_message_id: Option<Uuid>,
}

pub async fn insert_idempotent(
    tx: &mut Transaction<'_, Postgres>,
    input: NewMessage<'_>,
) -> Result<(Message, bool), AppError> {
    let NewMessage {
        message_id,
        client_message_id,
        conversation_id,
        sender_id,
        content,
        content_type,
        reply_to_message_id,
        forwarded_from_message_id,
    } = input;
    let inserted = sqlx::query_as::<_, Message>(
        "INSERT INTO messages
         (id,client_message_id,conversation_id,sender_id,content,content_type,reply_to_message_id,forwarded_from_message_id)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
         ON CONFLICT (conversation_id,client_message_id) DO NOTHING
         RETURNING id,client_message_id,conversation_id,sender_id,content,content_type,reply_to_message_id,forwarded_from_message_id,edited_at,deleted_at,created_at",
    )
    .bind(message_id)
    .bind(client_message_id)
    .bind(conversation_id)
    .bind(sender_id)
    .bind(content)
    .bind(content_type)
    .bind(reply_to_message_id)
    .bind(forwarded_from_message_id)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(message) = inserted {
        return Ok((message, true));
    }

    let existing = sqlx::query_as::<_, Message>(
        "SELECT id,client_message_id,conversation_id,sender_id,content,content_type,reply_to_message_id,forwarded_from_message_id,edited_at,deleted_at,created_at
         FROM messages WHERE conversation_id=$1 AND client_message_id=$2",
    )
    .bind(conversation_id)
    .bind(client_message_id)
    .fetch_one(&mut **tx)
    .await?;

    Ok((existing, false))
}

pub async fn find(pool: &PgPool, message_id: Uuid) -> Result<Option<Message>, AppError> {
    Ok(sqlx::query_as::<_, Message>(
        "SELECT id,client_message_id,conversation_id,sender_id,content,content_type,reply_to_message_id,forwarded_from_message_id,edited_at,deleted_at,created_at
         FROM messages WHERE id=$1",
    )
    .bind(message_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn history(
    pool: &PgPool,
    conversation_id: Uuid,
    limit: i64,
    cursor: Option<(DateTime<Utc>, Uuid)>,
) -> Result<Vec<Message>, AppError> {
    if let Some((timestamp, id)) = cursor {
        Ok(sqlx::query_as::<_, Message>(
            "SELECT id,client_message_id,conversation_id,sender_id,content,content_type,reply_to_message_id,forwarded_from_message_id,edited_at,deleted_at,created_at
             FROM messages
             WHERE conversation_id=$1 AND (created_at,id)<($2,$3)
             ORDER BY created_at DESC,id DESC LIMIT $4",
        )
        .bind(conversation_id).bind(timestamp).bind(id).bind(limit)
        .fetch_all(pool).await?)
    } else {
        Ok(sqlx::query_as::<_, Message>(
            "SELECT id,client_message_id,conversation_id,sender_id,content,content_type,reply_to_message_id,forwarded_from_message_id,edited_at,deleted_at,created_at
             FROM messages WHERE conversation_id=$1
             ORDER BY created_at DESC,id DESC LIMIT $2",
        )
        .bind(conversation_id).bind(limit)
        .fetch_all(pool).await?)
    }
}

pub async fn edit(
    tx: &mut Transaction<'_, Postgres>,
    message_id: Uuid,
    old_content: &str,
    new_content: &str,
    edited_at: DateTime<Utc>,
) -> Result<Message, AppError> {
    sqlx::query(
        "INSERT INTO message_edits (id,message_id,old_content,edited_at) VALUES ($1,$2,$3,$4)",
    )
    .bind(Uuid::new_v4())
    .bind(message_id)
    .bind(old_content)
    .bind(edited_at)
    .execute(&mut **tx)
    .await?;
    Ok(sqlx::query_as::<_, Message>(
        "UPDATE messages SET content=$2,edited_at=$3 WHERE id=$1
         RETURNING id,client_message_id,conversation_id,sender_id,content,content_type,reply_to_message_id,forwarded_from_message_id,edited_at,deleted_at,created_at",
    )
    .bind(message_id).bind(new_content).bind(edited_at)
    .fetch_one(&mut **tx).await?)
}

pub async fn delete(
    tx: &mut Transaction<'_, Postgres>,
    message_id: Uuid,
    deleted_at: DateTime<Utc>,
) -> Result<Message, AppError> {
    Ok(sqlx::query_as::<_, Message>(
        "UPDATE messages SET content=NULL,deleted_at=$2 WHERE id=$1
         RETURNING id,client_message_id,conversation_id,sender_id,content,content_type,reply_to_message_id,forwarded_from_message_id,edited_at,deleted_at,created_at",
    )
    .bind(message_id).bind(deleted_at)
    .fetch_one(&mut **tx).await?)
}

pub async fn record_delivered(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    message_ids: &[Uuid],
) -> Result<Vec<(Uuid, Uuid, DateTime<Utc>)>, AppError> {
    let valid_count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*)
         FROM messages m
         JOIN conversation_members cm
           ON cm.conversation_id=m.conversation_id
          AND cm.user_id=$1
          AND cm.left_at IS NULL
         WHERE m.id = ANY($2)",
    )
    .bind(user_id)
    .bind(message_ids)
    .fetch_one(&mut **tx)
    .await?;

    if valid_count != message_ids.len() as i64 {
        return Err(AppError::NotFound(
            "One or more messages were not found or are not accessible".into(),
        ));
    }

    Ok(sqlx::query_as::<_, (Uuid, Uuid, DateTime<Utc>)>(
        "WITH inserted AS (
            INSERT INTO message_receipts (message_id,user_id,delivered_at)
            SELECT m.id,$1,NOW()
            FROM messages m
            WHERE m.id = ANY($2)
              AND m.sender_id <> $1
            ON CONFLICT (message_id,user_id) DO NOTHING
            RETURNING message_id, delivered_at
         )
         SELECT i.message_id,m.sender_id,i.delivered_at
         FROM inserted i
         JOIN messages m ON m.id=i.message_id",
    )
    .bind(user_id)
    .bind(message_ids)
    .fetch_all(&mut **tx)
    .await?)
}

pub async fn record_read(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    conversation_id: Uuid,
    last_read_message_id: Uuid,
) -> Result<Vec<(Uuid, Uuid, DateTime<Utc>)>, AppError> {
    let member = sqlx::query_scalar::<_, Option<Uuid>>(
        "SELECT last_read_message_id
         FROM conversation_members
         WHERE conversation_id=$1 AND user_id=$2 AND left_at IS NULL
         FOR UPDATE",
    )
    .bind(conversation_id)
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::Forbidden("You are not a member of this conversation".into()))?;

    let target = sqlx::query_as::<_, (Uuid, DateTime<Utc>)>(
        "SELECT id,created_at
         FROM messages
         WHERE id=$1 AND conversation_id=$2",
    )
    .bind(last_read_message_id)
    .bind(conversation_id)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Read target message not found".into()))?;

    if let Some(current_id) = member {
        let current = sqlx::query_as::<_, (DateTime<Utc>, Uuid)>(
            "SELECT created_at,id FROM messages WHERE id=$1",
        )
        .bind(current_id)
        .fetch_optional(&mut **tx)
        .await?;

        if let Some(current) = current {
            if (target.1, target.0) <= current {
                return Ok(Vec::new());
            }
        }
    }

    let statuses = sqlx::query_as::<_, (Uuid, Uuid, DateTime<Utc>)>(
        "WITH inserted AS (
            INSERT INTO message_receipts (message_id,user_id,delivered_at,read_at)
            SELECT m.id,$1,NOW(),NOW()
            FROM messages m
            WHERE m.conversation_id=$2
              AND m.sender_id <> $1
              AND (m.created_at,m.id) <= ($3,$4)
            ON CONFLICT (message_id,user_id) DO UPDATE
              SET read_at=COALESCE(message_receipts.read_at,EXCLUDED.read_at),
                  delivered_at=COALESCE(message_receipts.delivered_at,EXCLUDED.delivered_at)
              WHERE message_receipts.read_at IS NULL
            RETURNING message_id,read_at
         )
         SELECT i.message_id,m.sender_id,i.read_at
         FROM inserted i
         JOIN messages m ON m.id=i.message_id",
    )
    .bind(user_id)
    .bind(conversation_id)
    .bind(target.1)
    .bind(target.0)
    .fetch_all(&mut **tx)
    .await?;

    sqlx::query(
        "UPDATE conversation_members
         SET last_read_message_id=$3
         WHERE conversation_id=$1 AND user_id=$2",
    )
    .bind(conversation_id)
    .bind(user_id)
    .bind(target.0)
    .execute(&mut **tx)
    .await?;

    Ok(statuses)
}

pub async fn add_reaction(
    tx: &mut Transaction<'_, Postgres>,
    message_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        "INSERT INTO message_reactions (message_id,user_id,emoji) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING",
    ).bind(message_id).bind(user_id).bind(emoji).execute(&mut **tx).await?;
    Ok(result.rows_affected() > 0)
}

pub async fn remove_reaction(
    tx: &mut Transaction<'_, Postgres>,
    message_id: Uuid,
    user_id: Uuid,
    emoji: &str,
) -> Result<bool, AppError> {
    let result = sqlx::query(
        "DELETE FROM message_reactions WHERE message_id=$1 AND user_id=$2 AND emoji=$3",
    )
    .bind(message_id)
    .bind(user_id)
    .bind(emoji)
    .execute(&mut **tx)
    .await?;
    Ok(result.rows_affected() > 0)
}
