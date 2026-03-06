//! Créditos del usuario.

use axum::{
    extract::State,
    response::Json,
    Json as JsonExtract,
};
use crate::models::{AddCreditsRequest, UserCredits};
use crate::repository;
use crate::AppState;

/// GET /api/credits
pub async fn get_credits(State(state): State<AppState>) -> Result<Json<UserCredits>, (axum::http::StatusCode, String)> {
    let row = repository::get_credits(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("get_credits: {}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;
    Ok(Json(UserCredits {
        id: row.id,
        balance: row.balance,
        updated_at: row.updated_at,
    }))
}

/// POST /api/credits/add
pub async fn add_credits(
    State(state): State<AppState>,
    JsonExtract(body): JsonExtract<AddCreditsRequest>,
) -> Result<Json<UserCredits>, (axum::http::StatusCode, String)> {
    let amount = body.amount.max(0);
    let row = repository::add_credits(&state.pool, amount)
        .await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(UserCredits {
        id: row.id,
        balance: row.balance,
        updated_at: row.updated_at,
    }))
}
