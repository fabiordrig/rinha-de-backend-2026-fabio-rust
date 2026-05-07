use crate::{
    scoring::ScoringEngine,
    types::{ScoreRequest, ScoreResponse},
};

pub fn score_request_with_fallback(
    engine: &ScoringEngine,
    request: &ScoreRequest,
) -> ScoreResponse {
    match engine.score(request) {
        Ok(response) => response,
        Err(error) => {
            tracing::error!(error = %error, transaction_id = %request.id, "failed to score request");
            ScoreResponse {
                approved: true,
                fraud_score: 0.0,
            }
        }
    }
}
