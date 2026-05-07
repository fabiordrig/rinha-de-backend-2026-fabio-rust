use rinha_de_backend_2026_fabio_rust::{
    hot_path::score_request_with_fallback,
    scoring::ScoringEngine,
    types::{
        Customer, LastTransaction, Merchant, ScoreRequest, Terminal, Transaction,
    },
};

fn sample_request(requested_at: &str) -> ScoreRequest {
    ScoreRequest {
        id: "tx-123".to_string(),
        transaction: Transaction {
            amount: 384.88,
            installments: 3,
            requested_at: requested_at.to_string(),
        },
        customer: Customer {
            avg_amount: 769.76,
            tx_count_24h: 3,
            known_merchants: vec!["MERC-009".to_string(), "MERC-001".to_string()],
        },
        merchant: Merchant {
            id: "MERC-001".to_string(),
            mcc: "5912".to_string(),
            avg_amount: 298.95,
        },
        terminal: Terminal {
            is_online: false,
            card_present: true,
            km_from_home: 13.7,
        },
        last_transaction: Some(LastTransaction {
            timestamp: "2026-03-11T14:58:35Z".to_string(),
            km_from_current: 18.8,
        }),
    }
}

#[test]
fn score_request_with_fallback_returns_engine_result_when_scoring_succeeds() {
    let engine = ScoringEngine::empty();

    let response = score_request_with_fallback(&engine, &sample_request("2026-03-11T20:23:35Z"));

    assert!(response.approved);
    assert_eq!(response.fraud_score, 0.0);
}

#[test]
fn score_request_with_fallback_returns_safe_fallback_when_scoring_errors() {
    let engine = ScoringEngine::empty();

    let response = score_request_with_fallback(&engine, &sample_request("not-a-timestamp"));

    assert!(response.approved);
    assert_eq!(response.fraud_score, 0.0);
}
