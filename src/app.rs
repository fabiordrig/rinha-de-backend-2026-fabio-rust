use axum::{routing::{get, post}, Json, Router};

use crate::types::{ScoreRequest, ScoreResponse};

pub fn build_app() -> Router {
    Router::new()
        .route("/ready", get(ready))
        .route("/fraud-score", post(fraud_score))
}

async fn ready() -> &'static str {
    "ok"
}

async fn fraud_score(Json(_request): Json<ScoreRequest>) -> Json<ScoreResponse> {
    Json(ScoreResponse {
        approved: true,
        fraud_score: 0.0,
    })
}
