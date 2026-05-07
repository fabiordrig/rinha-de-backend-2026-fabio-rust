use std::collections::HashMap;

use chrono::{DateTime, Datelike, Timelike, Utc};

use crate::types::{Normalization, ScoreRequest};

#[derive(Debug, thiserror::Error)]
pub enum VectorizeError {
    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),
}

pub fn vectorize(
    request: &ScoreRequest,
    normalization: &Normalization,
    mcc_risk: &HashMap<String, f64>,
) -> Result<[f64; 14], VectorizeError> {
    let requested_at = parse_utc(&request.transaction.requested_at)?;

    let (minutes_since_last_tx, km_from_last_tx) = match &request.last_transaction {
        Some(last_transaction) => {
            let last_timestamp = parse_utc(&last_transaction.timestamp)?;
            let minutes = (requested_at - last_timestamp).num_seconds() as f64 / 60.0;
            (
                round4(clamp(minutes / normalization.max_minutes)),
                round4(clamp(last_transaction.km_from_current / normalization.max_km)),
            )
        }
        None => (-1.0, -1.0),
    };

    let amount_vs_avg = if request.customer.avg_amount <= 0.0 {
        1.0
    } else {
        clamp(
            (request.transaction.amount / request.customer.avg_amount)
                / normalization.amount_vs_avg_ratio,
        )
    };

    let unknown_merchant = !request
        .customer
        .known_merchants
        .iter()
        .any(|merchant| merchant == &request.merchant.id);

    Ok([
        round4(clamp(request.transaction.amount / normalization.max_amount)),
        round4(clamp(
            request.transaction.installments as f64 / normalization.max_installments,
        )),
        round4(amount_vs_avg),
        round4(requested_at.hour() as f64 / 23.0),
        round4(requested_at.weekday().num_days_from_monday() as f64 / 6.0),
        minutes_since_last_tx,
        km_from_last_tx,
        round4(clamp(request.terminal.km_from_home / normalization.max_km)),
        round4(clamp(
            request.customer.tx_count_24h as f64 / normalization.max_tx_count_24h,
        )),
        bool_to_f64(request.terminal.is_online),
        bool_to_f64(request.terminal.card_present),
        bool_to_f64(unknown_merchant),
        round4(*mcc_risk.get(&request.merchant.mcc).unwrap_or(&0.5)),
        round4(clamp(
            request.merchant.avg_amount / normalization.max_merchant_avg_amount,
        )),
    ])
}

fn parse_utc(input: &str) -> Result<DateTime<Utc>, VectorizeError> {
    DateTime::parse_from_rfc3339(input)
        .map(|timestamp| timestamp.with_timezone(&Utc))
        .map_err(|_| VectorizeError::InvalidTimestamp(input.to_string()))
}

fn clamp(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn round4(value: f64) -> f64 {
    (value * 10_000.0).round() / 10_000.0
}

fn bool_to_f64(value: bool) -> f64 {
    if value {
        1.0
    } else {
        0.0
    }
}
