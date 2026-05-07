use axum::{body::Body, http::{Request, StatusCode}};
use hyper::body::to_bytes;
use tower::ServiceExt;

use rinha_de_backend_2026_fabio_rust::app::build_app;

#[tokio::test]
async fn ready_returns_ok() {
    let app = build_app();

    let response = app
        .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn fraud_score_returns_json_contract() {
    let app = build_app();
    let payload = r#"{
      "id": "tx-123",
      "transaction": { "amount": 384.88, "installments": 3, "requested_at": "2026-03-11T20:23:35Z" },
      "customer": { "avg_amount": 769.76, "tx_count_24h": 3, "known_merchants": ["MERC-009", "MERC-001"] },
      "merchant": { "id": "MERC-001", "mcc": "5912", "avg_amount": 298.95 },
      "terminal": { "is_online": false, "card_present": true, "km_from_home": 13.7 },
      "last_transaction": { "timestamp": "2026-03-11T14:58:35Z", "km_from_current": 18.8 }
    }"#;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/fraud-score")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body()).await.unwrap();
    let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert!(body_json.get("approved").unwrap().is_boolean());
    assert!(body_json.get("fraud_score").unwrap().is_number());
}
