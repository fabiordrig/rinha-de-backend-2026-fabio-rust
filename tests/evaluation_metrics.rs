use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use rinha_de_backend_2026_fabio_rust::{
    resources::{LoadedReferenceSource, LoadedResources},
    scoring::{
        evaluate_entries, load_evaluation_entries_from_file, EvaluationEntry, ScoringEngine,
    },
    types::{
        Customer, Merchant, Normalization, ReferenceLabel, ReferenceRecord, ScoreRequest, Terminal,
        Transaction,
    },
};

fn request(id: &str, amount: f64, merchant_id: &str, mcc: &str) -> ScoreRequest {
    ScoreRequest {
        id: id.to_string(),
        transaction: Transaction {
            amount,
            installments: 1,
            requested_at: "2026-03-11T12:00:00Z".to_string(),
        },
        customer: Customer {
            avg_amount: 100.0,
            tx_count_24h: 1,
            known_merchants: vec![merchant_id.to_string()],
        },
        merchant: Merchant {
            id: merchant_id.to_string(),
            mcc: mcc.to_string(),
            avg_amount: 100.0,
        },
        terminal: Terminal {
            is_online: false,
            card_present: true,
            km_from_home: 1.0,
        },
        last_transaction: None,
    }
}

#[test]
fn load_evaluation_entries_from_file_reads_official_shape() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("rinha-eval-{unique}.json"));

    fs::write(
        &path,
        r#"{
          "references_checksum_sha256": "abc",
          "stats": {"total": 1},
          "entries": [
            {
              "request": {
                "id": "tx-1",
                "transaction": {"amount": 10.0, "installments": 1, "requested_at": "2026-03-11T12:00:00Z"},
                "customer": {"avg_amount": 5.0, "tx_count_24h": 1, "known_merchants": ["MERC-1"]},
                "merchant": {"id": "MERC-1", "mcc": "5912", "avg_amount": 5.0},
                "terminal": {"is_online": false, "card_present": true, "km_from_home": 1.0},
                "last_transaction": null
              },
              "expected_approved": true,
              "expected_fraud_score": 0
            }
          ]
        }"#,
    )
    .unwrap();

    let entries = load_evaluation_entries_from_file(&path).unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].request.id, "tx-1");
    assert!(entries[0].expected_approved);
    assert_eq!(entries[0].expected_fraud_score, 0.0);
}

#[test]
fn evaluate_entries_reports_accuracy_and_confusion_matrix() {
    let resources = LoadedResources {
        normalization: Normalization {
            max_amount: 10_000.0,
            max_installments: 12.0,
            amount_vs_avg_ratio: 10.0,
            max_minutes: 1_440.0,
            max_km: 1_000.0,
            max_tx_count_24h: 20.0,
            max_merchant_avg_amount: 10_000.0,
        },
        mcc_risk: [("5912".to_string(), 0.2), ("7801".to_string(), 0.9)]
            .into_iter()
            .collect(),
        references: LoadedReferenceSource::Owned(vec![
            ReferenceRecord {
                vector: [
                    0.01, 0.1, 0.5, 0.5, 0.2, -1.0, -1.0, 0.001, 0.05, 0.0, 1.0, 0.0, 0.2,
                    0.01,
                ],
                label: ReferenceLabel::Legit,
            },
            ReferenceRecord {
                vector: [
                    0.01, 0.1, 0.5, 0.5, 0.2, -1.0, -1.0, 0.001, 0.05, 0.0, 1.0, 0.0, 0.2,
                    0.01,
                ],
                label: ReferenceLabel::Legit,
            },
            ReferenceRecord {
                vector: [
                    0.01, 0.1, 0.5, 0.5, 0.2, -1.0, -1.0, 0.001, 0.05, 0.0, 1.0, 0.0, 0.2,
                    0.01,
                ],
                label: ReferenceLabel::Legit,
            },
            ReferenceRecord {
                vector: [
                    0.01, 0.1, 0.5, 0.5, 0.2, -1.0, -1.0, 0.001, 0.05, 0.0, 1.0, 0.0, 0.2,
                    0.01,
                ],
                label: ReferenceLabel::Fraud,
            },
            ReferenceRecord {
                vector: [
                    0.01, 0.1, 0.5, 0.5, 0.2, -1.0, -1.0, 0.001, 0.05, 0.0, 1.0, 0.0, 0.2,
                    0.01,
                ],
                label: ReferenceLabel::Fraud,
            },
            ReferenceRecord {
                vector: [
                    0.50, 0.1, 0.5, 0.5, 0.2, -1.0, -1.0, 0.001, 0.05, 0.0, 1.0, 0.0, 0.9,
                    0.01,
                ],
                label: ReferenceLabel::Fraud,
            },
            ReferenceRecord {
                vector: [
                    0.50, 0.1, 0.5, 0.5, 0.2, -1.0, -1.0, 0.001, 0.05, 0.0, 1.0, 0.0, 0.9,
                    0.01,
                ],
                label: ReferenceLabel::Fraud,
            },
            ReferenceRecord {
                vector: [
                    0.50, 0.1, 0.5, 0.5, 0.2, -1.0, -1.0, 0.001, 0.05, 0.0, 1.0, 0.0, 0.9,
                    0.01,
                ],
                label: ReferenceLabel::Fraud,
            },
            ReferenceRecord {
                vector: [
                    0.50, 0.1, 0.5, 0.5, 0.2, -1.0, -1.0, 0.001, 0.05, 0.0, 1.0, 0.0, 0.9,
                    0.01,
                ],
                label: ReferenceLabel::Legit,
            },
            ReferenceRecord {
                vector: [
                    0.50, 0.1, 0.5, 0.5, 0.2, -1.0, -1.0, 0.001, 0.05, 0.0, 1.0, 0.0, 0.9,
                    0.01,
                ],
                label: ReferenceLabel::Legit,
            },
        ]),
    };

    let engine = ScoringEngine::new(resources);
    let entries = vec![
        EvaluationEntry {
            request: request("legit-hit", 100.0, "MERC-001", "5912"),
            expected_approved: true,
            expected_fraud_score: 0.4,
        },
        EvaluationEntry {
            request: request("fraud-hit", 5000.0, "MERC-999", "7801"),
            expected_approved: false,
            expected_fraud_score: 0.6,
        },
        EvaluationEntry {
            request: request("forced-miss", 100.0, "MERC-001", "5912"),
            expected_approved: false,
            expected_fraud_score: 1.0,
        },
    ];

    let summary = evaluate_entries(&engine, &entries).unwrap();

    assert_eq!(summary.total, 3);
    assert_eq!(summary.correct, 2);
    assert_eq!(summary.true_positive, 1);
    assert_eq!(summary.true_negative, 1);
    assert_eq!(summary.false_positive, 0);
    assert_eq!(summary.false_negative, 1);
    assert!((summary.accuracy - (2.0 / 3.0)).abs() < 1e-9);
    assert!((summary.recall_fraud - 0.5).abs() < 1e-9);
    assert!((summary.precision_fraud - 1.0).abs() < 1e-9);
    assert!((summary.avg_score_error - (0.0 + 0.0 + 0.6) / 3.0).abs() < 1e-9);
}
