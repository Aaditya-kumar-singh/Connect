use crate::app_state::AppState;
use crate::routes::create_router;
use axum::Router;

pub fn build(state: AppState) -> Router {
    create_router(state)
}
