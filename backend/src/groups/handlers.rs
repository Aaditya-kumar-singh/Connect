use crate::{
    app_state::AppState,
    auth::models::AuthenticatedSession,
    domain::errors::AppError,
    groups::{models::*, service},
};
use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

pub async fn create(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Json(input): Json<CreateGroupRequest>,
) -> Result<Json<GroupDetail>, AppError> {
    Ok(Json(service::create(&state, &auth, &input).await?))
}

pub async fn detail(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(group_id): Path<Uuid>,
) -> Result<Json<GroupDetail>, AppError> {
    Ok(Json(service::detail(&state, &auth, group_id).await?))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(group_id): Path<Uuid>,
    Json(input): Json<UpdateGroupRequest>,
) -> Result<Json<GroupDetail>, AppError> {
    Ok(Json(
        service::update(&state, &auth, group_id, &input).await?,
    ))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(group_id): Path<Uuid>,
) -> Result<(), AppError> {
    service::delete(&state, &auth, group_id).await
}

pub async fn add_members(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(group_id): Path<Uuid>,
    Json(input): Json<AddMembersRequest>,
) -> Result<Json<GroupDetail>, AppError> {
    Ok(Json(
        service::add_members(&state, &auth, group_id, &input).await?,
    ))
}

pub async fn remove_member(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<GroupDetail>, AppError> {
    Ok(Json(
        service::remove_member(&state, &auth, group_id, user_id).await?,
    ))
}

pub async fn leave(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path(group_id): Path<Uuid>,
) -> Result<(), AppError> {
    service::leave(&state, &auth, group_id).await
}

pub async fn change_role(
    State(state): State<AppState>,
    auth: AuthenticatedSession,
    Path((group_id, user_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<ChangeRoleRequest>,
) -> Result<Json<GroupDetail>, AppError> {
    Ok(Json(
        service::change_role(&state, &auth, group_id, user_id, &input).await?,
    ))
}
