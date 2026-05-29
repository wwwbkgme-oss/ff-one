use crate::{handlers::*, state::AppState};
use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
pub fn build_app(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/world", get(get_world))
        .route("/world/tick", post(tick_world))
        .route("/agents", get(list_agents).post(spawn_agent))
        .route("/agents/{id}/command", post(command_agent))
        .route("/sandbox", post(create_sandbox))
        .route("/sandbox/{id}/execute", post(execute_code))
        .route("/security/assess", post(assess_code))
        .route("/quests", get(list_quests))
        .route("/quests/{id}/accept", post(accept_quest))
        .route("/economy/market", get(get_market))
        .route("/drivers", get(list_drivers))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
}
