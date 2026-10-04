use crate::{domain::errors::AppError, groups::models::*};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

pub async fn create(
    tx: &mut Transaction<'_, Postgres>,
    creator_id: Uuid,
    name: &str,
    description: Option<&str>,
    member_ids: &[Uuid],
) -> Result<GroupDetail, AppError> {
    let conversation_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO conversations (conversation_type) VALUES ('GROUP') RETURNING id",
    )
    .fetch_one(&mut **tx)
    .await?;

    let group = sqlx::query_as::<_, Group>(
        "INSERT INTO groups (conversation_id,name,description,created_by)
         VALUES ($1,$2,$3,$4)
         RETURNING id,conversation_id,name,description,avatar_url,created_by,created_at,updated_at",
    )
    .bind(conversation_id)
    .bind(name.trim())
    .bind(description.map(str::trim).filter(|v| !v.is_empty()))
    .bind(creator_id)
    .fetch_one(&mut **tx)
    .await?;

    for user_id in member_ids {
        sqlx::query(
            "INSERT INTO conversation_members (conversation_id,user_id)
             VALUES ($1,$2)",
        )
        .bind(conversation_id)
        .bind(user_id)
        .execute(&mut **tx)
        .await?;

        sqlx::query(
            "INSERT INTO group_members (group_id,user_id,role)
             VALUES ($1,$2,$3)",
        )
        .bind(group.id)
        .bind(user_id)
        .bind(if *user_id == creator_id {
            "ADMIN"
        } else {
            "MEMBER"
        })
        .execute(&mut **tx)
        .await?;
    }

    detail_tx(tx, group).await
}

async fn detail_tx(
    tx: &mut Transaction<'_, Postgres>,
    group: Group,
) -> Result<GroupDetail, AppError> {
    let members = sqlx::query_as::<_, GroupMember>(
        "SELECT gm.user_id,u.display_name,p.avatar_url,gm.role,gm.joined_at
         FROM group_members gm
         JOIN users u ON u.id=gm.user_id
         LEFT JOIN user_profiles p ON p.user_id=u.id
         WHERE gm.group_id=$1 AND u.status='ACTIVE'
         ORDER BY gm.joined_at ASC,gm.user_id ASC",
    )
    .bind(group.id)
    .fetch_all(&mut **tx)
    .await?;
    Ok(GroupDetail { group, members })
}

pub async fn detail(pool: &PgPool, user_id: Uuid, group_id: Uuid) -> Result<GroupDetail, AppError> {
    let group = sqlx::query_as::<_, Group>(
        "SELECT g.id,g.conversation_id,g.name,g.description,g.avatar_url,g.created_by,g.created_at,g.updated_at
         FROM groups g
         JOIN group_members gm ON gm.group_id=g.id AND gm.user_id=$1
         WHERE g.id=$2 AND g.deleted_at IS NULL",
    )
    .bind(user_id)
    .bind(group_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Group not found".into()))?;

    let members = members(pool, group_id).await?;
    Ok(GroupDetail { group, members })
}

pub async fn members(pool: &PgPool, group_id: Uuid) -> Result<Vec<GroupMember>, AppError> {
    Ok(sqlx::query_as::<_, GroupMember>(
        "SELECT gm.user_id,u.display_name,p.avatar_url,gm.role,gm.joined_at
         FROM group_members gm
         JOIN users u ON u.id=gm.user_id
         LEFT JOIN user_profiles p ON p.user_id=u.id
         WHERE gm.group_id=$1 AND u.status='ACTIVE'
         ORDER BY gm.joined_at ASC,gm.user_id ASC",
    )
    .bind(group_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_group(pool: &PgPool, group_id: Uuid) -> Result<Group, AppError> {
    sqlx::query_as::<_, Group>(
        "SELECT id,conversation_id,name,description,avatar_url,created_by,created_at,updated_at
         FROM groups WHERE id=$1 AND deleted_at IS NULL",
    )
    .bind(group_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("Group not found".into()))
}

pub async fn role(pool: &PgPool, group_id: Uuid, user_id: Uuid) -> Result<String, AppError> {
    sqlx::query_scalar::<_, String>(
        "SELECT role FROM group_members WHERE group_id=$1 AND user_id=$2",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Forbidden("You are not a group member".into()))
}

pub async fn member_exists(pool: &PgPool, group_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM group_members WHERE group_id=$1 AND user_id=$2)",
    )
    .bind(group_id)
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}

pub async fn active_group_count(pool: &PgPool, user_id: Uuid) -> Result<i64, AppError> {
    Ok(sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM group_members gm
         JOIN groups g ON g.id=gm.group_id
         WHERE gm.user_id=$1 AND g.deleted_at IS NULL",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?)
}
