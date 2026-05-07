use std::{fs, io::Write, path::PathBuf, sync::Arc, time::{SystemTime, UNIX_EPOCH}};

use axum::{body::Body, http::{Request, StatusCode}};
use flate2::{write::GzEncoder, Compression};
use hyper::body::to_bytes;
use tower::ServiceExt;

use rinha_de_backend_2026_fabio_rust::{
    app::build_app_with_engine,
    resources::load_resources_from_dir,
    scoring::ScoringEngine,
    types::ScoreRequest,
};

fn temp_fixture_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let dir = std::env::temp_dir().join(format!("rinha-runtime-scoring-{unique}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_gzip(path: &PathBuf, content: &str) {
    let file = fs::File::create(path).unwrap();
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(content.as_bytes()).unwrap();
    encoder.finish().unwrap();
}

fn sample_request() -> ScoreRequest {
    serde_json::from_str(
        r#"{
          "id": "tx-3576980410",
          "transaction": { "amount": 384.88, "installments": 3, "requested_at": "2026-03-11T20:23:35Z" },
          "customer": { "avg_amount": 769.76, "tx_count_24h": 3, "known_merchants": ["MERC-009", "MERC-001", "MERC-001"] },
          "merchant": { "id": "MERC-001", "mcc": "5912", "avg_amount": 298.95 },
          "terminal": { "is_online": false, "card_present": true, "km_from_home": 13.7090520965 },
          "last_transaction": { "timestamp": "2026-03-11T14:58:35Z", "km_from_current": 18.8626479774 }
        }"#,
    )
    .unwrap()
}

#[test]
fn loads_resources_and_scores_with_exact_knn() {
    let fixture_dir = temp_fixture_dir();

    fs::write(
        fixture_dir.join("normalization.json"),
        r#"{
          "max_amount": 10000,
          "max_installments": 12,
          "amount_vs_avg_ratio": 10,
          "max_minutes": 1440,
          "max_km": 1000,
          "max_tx_count_24h": 20,
          "max_merchant_avg_amount": 10000
        }"#,
    )
    .unwrap();

    fs::write(
        fixture_dir.join("mcc_risk.json"),
        r#"{
          "5912": 0.2
        }"#,
    )
    .unwrap();

    write_gzip(
        &fixture_dir.join("references.json.gz"),
        r#"[
          { "vector": [0.0385, 0.25, 0.05, 0.8696, 0.3333, 0.2257, 0.0189, 0.0137, 0.15, 0.0, 1.0, 0.0, 0.2, 0.0299], "label": "legit" },
          { "vector": [0.0384, 0.25, 0.05, 0.8695, 0.3333, 0.2256, 0.0189, 0.0136, 0.15, 0.0, 1.0, 0.0, 0.2, 0.0298], "label": "fraud" },
          { "vector": [0.0386, 0.25, 0.05, 0.8697, 0.3333, 0.2258, 0.0190, 0.0138, 0.15, 0.0, 1.0, 0.0, 0.2, 0.0300], "label": "fraud" },
          { "vector": [0.0385, 0.2501, 0.05, 0.8696, 0.3333, 0.2257, 0.0188, 0.0137, 0.15, 0.0, 1.0, 0.0, 0.2, 0.0299], "label": "legit" },
          { "vector": [0.0385, 0.2499, 0.05, 0.8696, 0.3333, 0.2257, 0.0189, 0.0137, 0.15, 0.0, 1.0, 0.0, 0.2, 0.0299], "label": "fraud" },
          { "vector": [0.9506, 0.8333, 1.0, 0.2174, 0.8333, -1.0, -1.0, 0.9523, 1.0, 0.0, 1.0, 1.0, 0.75, 0.0055], "label": "fraud" }
        ]"#,
    );

    let resources = load_resources_from_dir(&fixture_dir).unwrap();
    let engine = ScoringEngine::new(resources);
    let decision = engine.score(&sample_request()).unwrap();

    assert_eq!(decision.fraud_score, 0.6);
    assert!(!decision.approved);
}

#[tokio::test]
async fn fraud_score_endpoint_uses_loaded_engine() {
    let fixture_dir = temp_fixture_dir();

    fs::write(
        fixture_dir.join("normalization.json"),
        r#"{
          "max_amount": 10000,
          "max_installments": 12,
          "amount_vs_avg_ratio": 10,
          "max_minutes": 1440,
          "max_km": 1000,
          "max_tx_count_24h": 20,
          "max_merchant_avg_amount": 10000
        }"#,
    )
    .unwrap();

    fs::write(
        fixture_dir.join("mcc_risk.json"),
        r#"{
          "5912": 0.2
        }"#,
    )
    .unwrap();

    write_gzip(
        &fixture_dir.join("references.json.gz"),
        r#"[
          { "vector": [0.0385, 0.25, 0.05, 0.8696, 0.3333, 0.2257, 0.0189, 0.0137, 0.15, 0.0, 1.0, 0.0, 0.2, 0.0299], "label": "legit" },
          { "vector": [0.0384, 0.25, 0.05, 0.8695, 0.3333, 0.2256, 0.0189, 0.0136, 0.15, 0.0, 1.0, 0.0, 0.2, 0.0298], "label": "fraud" },
          { "vector": [0.0386, 0.25, 0.05, 0.8697, 0.3333, 0.2258, 0.0190, 0.0138, 0.15, 0.0, 1.0, 0.0, 0.2, 0.0300], "label": "fraud" },
          { "vector": [0.0385, 0.2501, 0.05, 0.8696, 0.3333, 0.2257, 0.0188, 0.0137, 0.15, 0.0, 1.0, 0.0, 0.2, 0.0299], "label": "legit" },
          { "vector": [0.0385, 0.2499, 0.05, 0.8696, 0.3333, 0.2257, 0.0189, 0.0137, 0.15, 0.0, 1.0, 0.0, 0.2, 0.0299], "label": "fraud" }
        ]"#,
    );

    let resources = load_resources_from_dir(&fixture_dir).unwrap();
    let engine = Arc::new(ScoringEngine::new(resources));
    let app = build_app_with_engine(engine);
    let payload = serde_json::to_vec(&sample_request()).unwrap();

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

    assert_eq!(body_json.get("approved").unwrap(), &serde_json::Value::Bool(false));
    assert_eq!(body_json.get("fraud_score").unwrap(), &serde_json::json!(0.6));
}
