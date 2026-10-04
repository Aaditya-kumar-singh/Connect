use crate::{calls::models::*, domain::errors::AppError};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

fn user_lock_key(user_id: Uuid) -> String {
    format!("call-user:{user_id}")
}

pub async fn create(
    pool: &PgPool,
    caller_id: Uuid,
    callee_id: Uuid,
    conversation_id: Uuid,
    call_type: &str,
) -> Result<CallRecord, AppError> {
    let mut tx: Transaction<'_, Postgres> = pool.begin().await?;
    let (first_user, second_user) = if caller_id < callee_id {
        (caller_id, callee_id)
    } else {
        (callee_id, caller_id)
    };
    for user_id in [first_user, second_user] {
        sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
            .bind(user_lock_key(user_id))
            .execute(&mut *tx)
            .await?;
    }

    let active: bool = sqlx::query_scalar(
        "SELECT EXISTS(
            SELECT 1 FROM calls c
            JOIN call_participants cp ON cp.call_id=c.id
            WHERE cp.user_id = ANY($1::uuid[])
              AND c.status IN ('CALLING','RINGING','CONNECTING','CONNECTED')
        )",
    )
    .bind(vec![caller_id, callee_id])
    .fetch_one(&mut *tx)
    .await?;
    if active {
        return Err(AppError::Conflict(
            "User is already in an active call".into(),
        ));
    }

    let call = sqlx::query_as::<_, CallRecord>(
        "INSERT INTO calls (conversation_id,initiated_by,call_type,status)
         VALUES ($1,$2,$3,'CALLING')
         RETURNING id,conversation_id,initiated_by,call_type,status,started_at,ended_at,
                   duration_seconds,end_reason,created_at",
    )
    .bind(conversation_id)
    .bind(caller_id)
    .bind(call_type)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO call_participants (call_id,user_id) VALUES ($1,$2),($1,$3)")
        .bind(call.id)
        .bind(caller_id)
        .bind(callee_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(call)
}

pub async fn set_ringing(pool: &PgPool, call_id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query(
        "UPDATE calls SET status='RINGING'
         WHERE id=$1 AND status='CALLING'",
    )
    .bind(call_id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected() == 1)
}

pub async fn answer(pool: &PgPool, call_id: Uuid, user_id: Uuid) -> Result<CallRecord, AppError> {
    sqlx::query_as::<_, CallRecord>(
        "UPDATE calls c
         SET status='CONNECTING'
         WHERE c.id=$1 AND c.status='RINGING'
           AND EXISTS (SELECT 1 FROM call_participants p WHERE p.call_id=c.id AND p.user_id=$2)
           AND c.initiated_by<>$2
         RETURNING c.id,c.conversation_id,c.initiated_by,c.call_type,c.status,c.started_at,
                   c.ended_at,c.duration_seconds,c.end_reason,c.created_at",
    )
    .bind(call_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Conflict("Call is no longer answerable".into()))
}

pub async fn reject(pool: &PgPool, call_id: Uuid, user_id: Uuid) -> Result<CallRecord, AppError> {
    finish(pool, call_id, user_id, "REJECTED", "REJECTED", true).await
}

pub async fn end(
    pool: &PgPool,
    call_id: Uuid,
    user_id: Uuid,
    reason: &str,
) -> Result<CallRecord, AppError> {
    let end_reason = match reason {
        "NORMAL" | "ICE_FAILED" | "TIMEOUT" | "ERROR" => reason,
        _ => return Err(AppError::Validation("Invalid call end reason".into())),
    };
    finish(pool, call_id, user_id, "ENDED", end_reason, false).await
}

async fn finish(
    pool: &PgPool,
    call_id: Uuid,
    user_id: Uuid,
    status: &str,
    reason: &str,
    reject_only: bool,
) -> Result<CallRecord, AppError> {
    let query = if reject_only {
        "UPDATE calls c SET status=$3,ended_at=NOW(),end_reason=$4
         WHERE c.id=$1 AND c.status='RINGING'
           AND EXISTS (SELECT 1 FROM call_participants p WHERE p.call_id=c.id AND p.user_id=$2)
           AND c.initiated_by<>$2
         RETURNING c.id,c.conversation_id,c.initiated_by,c.call_type,c.status,c.started_at,c.ended_at,
                   c.duration_seconds,c.end_reason,c.created_at"
    } else {
        "UPDATE calls c SET status=$3,ended_at=NOW(),end_reason=$4,
             duration_seconds=CASE WHEN c.started_at IS NULL THEN NULL
               ELSE GREATEST(0,EXTRACT(EPOCH FROM (NOW()-c.started_at))::INTEGER) END
         WHERE c.id=$1 AND c.status IN ('CALLING','RINGING','CONNECTING','CONNECTED')
           AND EXISTS (SELECT 1 FROM call_participants p WHERE p.call_id=c.id AND p.user_id=$2)
         RETURNING c.id,c.conversation_id,c.initiated_by,c.call_type,c.status,c.started_at,c.ended_at,
                   c.duration_seconds,c.end_reason,c.created_at"
    };
    sqlx::query_as::<_, CallRecord>(query)
        .bind(call_id)
        .bind(user_id)
        .bind(status)
        .bind(reason)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::Conflict("Call is no longer active".into()))
}

pub async fn connected(pool: &PgPool, call_id: Uuid, user_id: Uuid) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "UPDATE calls SET status='CONNECTED',started_at=COALESCE(started_at,NOW())
         WHERE id=$1 AND status='CONNECTING'
           AND EXISTS (SELECT 1 FROM call_participants WHERE call_id=$1 AND user_id=$2)
         RETURNING id",
    )
    .bind(call_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Conflict("Call is not connecting".into()))
}

pub async fn relay_allowed(pool: &PgPool, call_id: Uuid, user_id: Uuid) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM call_participants
         WHERE call_id=$1 AND user_id<>$2
           AND EXISTS (
             SELECT 1 FROM calls c WHERE c.id=$1
               AND c.status IN ('CONNECTING','CONNECTED')
           )",
    )
    .bind(call_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Forbidden("Call signaling is not allowed".into()))
}

pub async fn relay_allowed_for_terminal(
    pool: &PgPool,
    call_id: Uuid,
    user_id: Uuid,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM call_participants WHERE call_id=$1 AND user_id<>$2 LIMIT 1",
    )
    .bind(call_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Forbidden("Call participant not found".into()))
}

pub async fn participants(pool: &PgPool, call_id: Uuid) -> Result<Vec<Uuid>, AppError> {
    Ok(sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM call_participants WHERE call_id=$1 ORDER BY user_id",
    )
    .bind(call_id)
    .fetch_all(pool)
    .await?)
}

pub async fn mark_missed(
    pool: &PgPool,
    call_id: Uuid,
) -> Result<Option<(CallRecord, Uuid)>, AppError> {
    let mut tx = pool.begin().await?;
    let call = sqlx::query_as::<_, CallRecord>(
        "UPDATE calls SET status='MISSED',ended_at=NOW(),end_reason='TIMEOUT'
         WHERE id=$1 AND status='RINGING'
         RETURNING id,conversation_id,initiated_by,call_type,status,started_at,ended_at,
                   duration_seconds,end_reason,created_at",
    )
    .bind(call_id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(call) = call else {
        return Ok(None);
    };
    let callee = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM call_participants WHERE call_id=$1 AND user_id<>$2 LIMIT 1",
    )
    .bind(call_id)
    .bind(call.initiated_by)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Some((call, callee)))
}

pub async fn history(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> Result<Vec<CallHistoryItem>, AppError> {
    Ok(sqlx::query_as::<_, CallHistoryItem>(
        "SELECT c.id,c.conversation_id,c.initiated_by,c.call_type,c.status,c.started_at,c.ended_at,
                c.duration_seconds,c.end_reason,c.created_at,
                other.id AS other_user_id,other.display_name AS other_display_name
         FROM calls c
         JOIN call_participants me ON me.call_id=c.id AND me.user_id=$1
         JOIN call_participants op ON op.call_id=c.id AND op.user_id<>$1
         JOIN users other ON other.id=op.user_id
         ORDER BY c.created_at DESC
         LIMIT $2",
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn conversation_for_call(
    pool: &PgPool,
    caller_id: Uuid,
    callee_id: Uuid,
) -> Result<Uuid, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        "SELECT c.id
         FROM conversations c
         JOIN conversation_members a ON a.conversation_id=c.id AND a.user_id=$1 AND a.left_at IS NULL
         JOIN conversation_members b ON b.conversation_id=c.id AND b.user_id=$2 AND b.left_at IS NULL
         WHERE c.conversation_type='DIRECT'
         LIMIT 1",
    )
    .bind(caller_id)
    .bind(callee_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Direct conversation not found".into()))
}

pub async fn caller_display_name(pool: &PgPool, user_id: Uuid) -> Result<String, AppError> {
    sqlx::query_scalar::<_, String>(
        "SELECT display_name FROM users WHERE id=$1 AND status='ACTIVE'",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".into()))
}
