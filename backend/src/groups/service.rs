use crate::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    contacts::repository as contact_repo,
    domain::errors::AppError,
    groups::{models::*, repository, validation},
};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

const MAX_GROUPS_PER_USER: i64 = 100;

async fn require_admin(
    state: &AppState,
    auth: &AuthenticatedSession,
    group_id: Uuid,
) -> Result<Group, AppError> {
    let group = repository::get_group(state.db(), group_id).await?;
    let role = repository::role(state.db(), group_id, auth.user_id).await?;
    if role != "ADMIN" {
        return Err(AppError::Forbidden("Admin role required".into()));
    }
    Ok(group)
}

fn normalize_members(creator_id: Uuid, member_ids: &[Uuid]) -> Result<Vec<Uuid>, AppError> {
    let mut members = Vec::with_capacity(member_ids.len() + 1);
    members.push(creator_id);
    for user_id in member_ids {
        if !members.contains(user_id) {
            members.push(*user_id);
        }
    }
    validation::validate_members(members.len())?;
    Ok(members)
}

pub async fn create(
    state: &AppState,
    auth: &AuthenticatedSession,
    input: &CreateGroupRequest,
) -> Result<GroupDetail, AppError> {
    validation::validate_name(&input.name)?;
    validation::validate_description(input.description.as_deref())?;

    let members = normalize_members(auth.user_id, &input.member_ids)?;
    if repository::active_group_count(state.db(), auth.user_id).await? >= MAX_GROUPS_PER_USER {
        return Err(AppError::Conflict("Maximum group limit reached".into()));
    }

    for user_id in &members {
        if !contact_repo::user_exists(state.db(), *user_id).await? {
            return Err(AppError::NotFound("User not found".into()));
        }
        if *user_id != auth.user_id
            && (contact_repo::is_blocked(state.db(), auth.user_id, *user_id).await?
                || contact_repo::is_blocked(state.db(), *user_id, auth.user_id).await?)
        {
            return Err(AppError::Forbidden("A group member is blocked".into()));
        }
    }

    let mut tx = state.db().begin().await?;
    let group = repository::create(
        &mut tx,
        auth.user_id,
        &input.name,
        input.description.as_deref(),
        &members,
    )
    .await?;
    let creator_name = display_name(&mut tx, auth.user_id).await?;
    insert_system_message(
        &mut tx,
        group.group.conversation_id,
        format!("{} created the group", creator_name),
    )
    .await?;
    tx.commit().await?;

    Ok(group)
}

pub async fn detail(
    state: &AppState,
    auth: &AuthenticatedSession,
    group_id: Uuid,
) -> Result<GroupDetail, AppError> {
    repository::detail(state.db(), auth.user_id, group_id).await
}

pub async fn update(
    state: &AppState,
    auth: &AuthenticatedSession,
    group_id: Uuid,
    input: &UpdateGroupRequest,
) -> Result<GroupDetail, AppError> {
    let _ = require_admin(state, auth, group_id).await?;
    if let Some(name) = &input.name {
        validation::validate_name(name)?;
    }
    validation::validate_description(input.description.as_deref())?;
    validation::validate_avatar_url(input.avatar_url.as_deref())?;

    let mut tx = state.db().begin().await?;
    let group = sqlx::query_as::<_, Group>(
        "UPDATE groups
         SET name=COALESCE($2,name),
             description=COALESCE($3,description),
             avatar_url=COALESCE($4,avatar_url),
             updated_at=NOW()
         WHERE id=$1 AND deleted_at IS NULL
         RETURNING id,conversation_id,name,description,avatar_url,created_by,created_at,updated_at",
    )
    .bind(group_id)
    .bind(input.name.as_deref().map(str::trim))
    .bind(input.description.as_deref().map(str::trim))
    .bind(input.avatar_url.as_deref())
    .fetch_one(&mut *tx)
    .await?;

    let actor = display_name(&mut tx, auth.user_id).await?;
    if input.name.is_some() {
        insert_system_message(
            &mut tx,
            group.conversation_id,
            format!("{} changed the group name to '{}'", actor, group.name),
        )
        .await?;
    }
    tx.commit().await?;
    repository::detail(state.db(), auth.user_id, group_id).await
}

pub async fn add_members(
    state: &AppState,
    auth: &AuthenticatedSession,
    group_id: Uuid,
    input: &AddMembersRequest,
) -> Result<GroupDetail, AppError> {
    let group = require_admin(state, auth, group_id).await?;
    if input.user_ids.is_empty() {
        return Err(AppError::Validation(
            "user_ids must contain at least one user".into(),
        ));
    }

    let current = repository::members(state.db(), group_id).await?;
    if current.len() + input.user_ids.len() > 256 {
        return Err(AppError::Validation(
            "A group cannot exceed 256 members".into(),
        ));
    }

    let actor_name = {
        let mut tx = state.db().begin().await?;
        display_name(&mut tx, auth.user_id).await?
    };

    let mut tx = state.db().begin().await?;
    for user_id in &input.user_ids {
        if repository::member_exists(state.db(), group_id, *user_id).await? {
            return Err(AppError::Conflict("User is already a group member".into()));
        }
        if !contact_repo::user_exists(state.db(), *user_id).await? {
            return Err(AppError::NotFound("User not found".into()));
        }
        if contact_repo::is_blocked(state.db(), *user_id, auth.user_id).await?
            || contact_repo::is_blocked(state.db(), auth.user_id, *user_id).await?
        {
            return Err(AppError::Forbidden("A group member is blocked".into()));
        }

        sqlx::query("INSERT INTO group_members (group_id,user_id,role) VALUES ($1,$2,'MEMBER')")
            .bind(group_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO conversation_members (conversation_id,user_id) VALUES ($1,$2)")
            .bind(group.conversation_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        let name = display_name(&mut tx, *user_id).await?;
        insert_system_message(
            &mut tx,
            group.conversation_id,
            format!("{} added {} to the group", actor_name, name),
        )
        .await?;
    }
    tx.commit().await?;
    repository::detail(state.db(), auth.user_id, group_id).await
}

pub async fn remove_member(
    state: &AppState,
    auth: &AuthenticatedSession,
    group_id: Uuid,
    user_id: Uuid,
) -> Result<GroupDetail, AppError> {
    let group = require_admin(state, auth, group_id).await?;
    if user_id == auth.user_id {
        return Err(AppError::Validation("Use leave to remove yourself".into()));
    }
    let target_role = repository::role(state.db(), group_id, user_id).await?;
    if target_role == "ADMIN" {
        return Err(AppError::Forbidden("Cannot remove another admin".into()));
    }

    let mut tx = state.db().begin().await?;
    let actor = display_name(&mut tx, auth.user_id).await?;
    let target = display_name(&mut tx, user_id).await?;
    sqlx::query("DELETE FROM group_members WHERE group_id=$1 AND user_id=$2")
        .bind(group_id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE conversation_members SET left_at=NOW() WHERE conversation_id=$1 AND user_id=$2 AND left_at IS NULL")
        .bind(group.conversation_id).bind(user_id).execute(&mut *tx).await?;
    insert_system_message(
        &mut tx,
        group.conversation_id,
        format!("{} removed {} from the group", actor, target),
    )
    .await?;
    tx.commit().await?;

    repository::detail(state.db(), auth.user_id, group_id).await
}

pub async fn leave(
    state: &AppState,
    auth: &AuthenticatedSession,
    group_id: Uuid,
) -> Result<(), AppError> {
    let group = repository::get_group(state.db(), group_id).await?;
    let role = repository::role(state.db(), group_id, auth.user_id).await?;
    let mut tx = state.db().begin().await?;
    let actor = display_name(&mut tx, auth.user_id).await?;

    sqlx::query("DELETE FROM group_members WHERE group_id=$1 AND user_id=$2")
        .bind(group_id)
        .bind(auth.user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE conversation_members SET left_at=NOW() WHERE conversation_id=$1 AND user_id=$2 AND left_at IS NULL")
        .bind(group.conversation_id).bind(auth.user_id).execute(&mut *tx).await?;

    if role == "ADMIN" {
        let admin_count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM group_members WHERE group_id=$1 AND role='ADMIN'",
        )
        .bind(group_id)
        .fetch_one(&mut *tx)
        .await?;
        if admin_count == 0 {
            let next = sqlx::query_as::<_, (Uuid, chrono::DateTime<chrono::Utc>)>(
                "SELECT user_id,joined_at FROM group_members WHERE group_id=$1 ORDER BY joined_at ASC LIMIT 1",
            )
            .bind(group_id).fetch_optional(&mut *tx).await?;
            if let Some((next_user, _)) = next {
                sqlx::query(
                    "UPDATE group_members SET role='ADMIN' WHERE group_id=$1 AND user_id=$2",
                )
                .bind(group_id)
                .bind(next_user)
                .execute(&mut *tx)
                .await?;
                let next_name = display_name(&mut tx, next_user).await?;
                insert_system_message(
                    &mut tx,
                    group.conversation_id,
                    format!("{} left the group; {} is now an admin", actor, next_name),
                )
                .await?;
            }
        } else {
            insert_system_message(
                &mut tx,
                group.conversation_id,
                format!("{} left the group", actor),
            )
            .await?;
        }
    } else {
        insert_system_message(
            &mut tx,
            group.conversation_id,
            format!("{} left the group", actor),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(())
}

pub async fn change_role(
    state: &AppState,
    auth: &AuthenticatedSession,
    group_id: Uuid,
    user_id: Uuid,
    input: &ChangeRoleRequest,
) -> Result<GroupDetail, AppError> {
    validation::validate_role(&input.role)?;
    let group = require_admin(state, auth, group_id).await?;
    let target_role = repository::role(state.db(), group_id, user_id).await?;

    if user_id == auth.user_id && input.role == "MEMBER" && target_role == "ADMIN" {
        let admins = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM group_members WHERE group_id=$1 AND role='ADMIN'",
        )
        .bind(group_id)
        .fetch_one(state.db())
        .await?;
        if admins <= 1 {
            return Err(AppError::Conflict("Group must always have an admin".into()));
        }
    }

    let mut tx = state.db().begin().await?;
    sqlx::query("UPDATE group_members SET role=$3 WHERE group_id=$1 AND user_id=$2")
        .bind(group_id)
        .bind(user_id)
        .bind(&input.role)
        .execute(&mut *tx)
        .await?;
    let actor = display_name(&mut tx, auth.user_id).await?;
    let target = display_name(&mut tx, user_id).await?;
    let message = if input.role == "ADMIN" {
        format!("{} made {} an admin", actor, target)
    } else {
        format!("{} removed {} as an admin", actor, target)
    };
    insert_system_message(&mut tx, group.conversation_id, message).await?;
    tx.commit().await?;

    repository::detail(state.db(), auth.user_id, group_id).await
}

pub async fn delete(
    state: &AppState,
    auth: &AuthenticatedSession,
    group_id: Uuid,
) -> Result<(), AppError> {
    let group = require_admin(state, auth, group_id).await?;
    let mut tx = state.db().begin().await?;
    sqlx::query("UPDATE groups SET deleted_at=NOW(),updated_at=NOW() WHERE id=$1")
        .bind(group_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE conversation_members SET left_at=NOW() WHERE conversation_id=$1 AND left_at IS NULL")
        .bind(group.conversation_id).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(())
}

async fn display_name(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
) -> Result<String, AppError> {
    sqlx::query_scalar::<_, String>("SELECT display_name FROM users WHERE id=$1")
        .bind(user_id)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))
}

async fn insert_system_message(
    tx: &mut Transaction<'_, Postgres>,
    conversation_id: Uuid,
    content: String,
) -> Result<crate::messaging::models::Message, AppError> {
    Ok(sqlx::query_as::<_, crate::messaging::models::Message>(
        "INSERT INTO messages
         (id,client_message_id,conversation_id,sender_id,content,content_type)
         VALUES ($1,$2,$3,NULL,$4,'system')
         RETURNING id,client_message_id,conversation_id,sender_id,content,content_type,
                   reply_to_message_id,forwarded_from_message_id,edited_at,deleted_at,created_at",
    )
    .bind(Uuid::new_v4())
    .bind(Uuid::new_v4())
    .bind(conversation_id)
    .bind(content)
    .fetch_one(&mut **tx)
    .await?)
}
