use std::sync::Arc;

use axum::{extract::State, routing::{get, post}, Json, Router};

use crate::{
    hot_path::score_request_with_fallback,
    scoring::{ScoringEngine, SharedScoringEngine},
    types::{ScoreRequest, ScoreResponse},
};

pub fn build_app() -> Router {
    build_app_with_engine(Arc::new(ScoringEngine::empty()))
}

pub fn build_app_with_engine(engine: SharedScoringEngine) -> Router {
    Router::new()
        .route("/ready", get(ready))
        .route("/fraud-score", post(fraud_score))
        .with_state(engine)
}

async fn ready() -> &'static str {
    "ok"
}

async fn fraud_score(
    State(engine): State<SharedScoringEngine>,
    Json(request): Json<ScoreRequest>,
) -> Json<ScoreResponse> {
    Json(score_request_with_fallback(engine.as_ref(), &request))
}
