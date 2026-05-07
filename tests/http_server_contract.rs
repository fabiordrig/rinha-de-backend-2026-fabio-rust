use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use hyper::body::to_bytes;
use tower::ServiceExt;

use rinha_de_backend_2026_fabio_rust::app::build_app;

fn valid_payload() -> &'static str {
    r#"{
      "id": "tx-123",
      "transaction": { "amount": 384.88, "installments": 3, "requested_at": "2026-03-11T20:23:35Z" },
      "customer": { "avg_amount": 769.76, "tx_count_24h": 3, "known_merchants": ["MERC-009", "MERC-001"] },
      "merchant": { "id": "MERC-001", "mcc": "5912", "avg_amount": 298.95 },
      "terminal": { "is_online": false, "card_present": true, "km_from_home": 13.7 },
      "last_transaction": { "timestamp": "2026-03-11T14:58:35Z", "km_from_current": 18.8 }
    }"#
}

#[tokio::test]
async fn ready_returns_ok_text_body() {
    let app = build_app();

    let response = app
        .oneshot(Request::builder().uri("/ready").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"ok");
}

#[tokio::test]
async fn fraud_score_returns_200_with_json_contract() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/fraud-score")
                .header("content-type", "application/json")
                .body(Body::from(valid_payload()))
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

#[tokio::test]
async fn invalid_json_does_not_return_5xx() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/fraud-score")
                .header("content-type", "application/json")
                .body(Body::from("{not-json"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn get_on_fraud_score_does_not_return_5xx() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/fraud-score")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn unknown_route_returns_client_error() {
    let app = build_app();

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_client_error());
}
