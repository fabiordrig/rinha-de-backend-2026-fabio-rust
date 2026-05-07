use std::collections::HashMap;

use rinha_de_backend_2026_fabio_rust::types::{Customer, LastTransaction, Merchant, Normalization, ScoreRequest, Terminal, Transaction};
use rinha_de_backend_2026_fabio_rust::vectorize::vectorize;

fn normalization() -> Normalization {
    Normalization {
        max_amount: 10_000.0,
        max_installments: 12.0,
        amount_vs_avg_ratio: 10.0,
        max_minutes: 1_440.0,
        max_km: 1_000.0,
        max_tx_count_24h: 20.0,
        max_merchant_avg_amount: 10_000.0,
    }
}

#[test]
fn vectorizes_transaction_with_last_transaction_null_using_sentinel_values() {
    let request = ScoreRequest {
        id: "tx-1329056812".to_string(),
        transaction: Transaction {
            amount: 41.12,
            installments: 2,
            requested_at: "2026-03-11T18:45:53Z".to_string(),
        },
        customer: Customer {
            avg_amount: 82.24,
            tx_count_24h: 3,
            known_merchants: vec!["MERC-003".to_string(), "MERC-016".to_string()],
        },
        merchant: Merchant {
            id: "MERC-016".to_string(),
            mcc: "5411".to_string(),
            avg_amount: 60.25,
        },
        terminal: Terminal {
            is_online: false,
            card_present: true,
            km_from_home: 29.23,
        },
        last_transaction: None,
    };

    let mcc_risk = HashMap::from([("5411".to_string(), 0.15)]);

    let vector = vectorize(&request, &normalization(), &mcc_risk).unwrap();

    assert_eq!(
        vector,
        [
            0.0041,
            0.1667,
            0.05,
            0.7826,
            0.3333,
            -1.0,
            -1.0,
            0.0292,
            0.15,
            0.0,
            1.0,
            0.0,
            0.15,
            0.0060,
        ]
    );
}

#[test]
fn vectorizes_transaction_with_previous_transaction_and_unknown_mcc_default() {
    let request = ScoreRequest {
        id: "tx-3576980410".to_string(),
        transaction: Transaction {
            amount: 384.88,
            installments: 3,
            requested_at: "2026-03-11T20:23:35Z".to_string(),
        },
        customer: Customer {
            avg_amount: 769.76,
            tx_count_24h: 3,
            known_merchants: vec!["MERC-009".to_string(), "MERC-001".to_string(), "MERC-001".to_string()],
        },
        merchant: Merchant {
            id: "MERC-777".to_string(),
            mcc: "9999".to_string(),
            avg_amount: 298.95,
        },
        terminal: Terminal {
            is_online: false,
            card_present: true,
            km_from_home: 13.7090520965,
        },
        last_transaction: Some(LastTransaction {
            timestamp: "2026-03-11T14:58:35Z".to_string(),
            km_from_current: 18.8626479774,
        }),
    };

    let vector = vectorize(&request, &normalization(), &HashMap::new()).unwrap();

    assert_eq!(
        vector,
        [
            0.0385,
            0.25,
            0.05,
            0.8696,
            0.3333,
            0.2257,
            0.0189,
            0.0137,
            0.15,
            0.0,
            1.0,
            1.0,
            0.5,
            0.0299,
        ]
    );
}

#[test]
fn clamps_large_values_and_maps_boolean_fields() {
    let request = ScoreRequest {
        id: "tx-clamp".to_string(),
        transaction: Transaction {
            amount: 100_000.0,
            installments: 99,
            requested_at: "2026-03-15T00:10:00Z".to_string(),
        },
        customer: Customer {
            avg_amount: 1.0,
            tx_count_24h: 999,
            known_merchants: vec![],
        },
        merchant: Merchant {
            id: "MERC-XYZ".to_string(),
            mcc: "7802".to_string(),
            avg_amount: 100_000.0,
        },
        terminal: Terminal {
            is_online: true,
            card_present: false,
            km_from_home: 10_000.0,
        },
        last_transaction: Some(LastTransaction {
            timestamp: "2026-03-10T00:10:00Z".to_string(),
            km_from_current: 10_000.0,
        }),
    };

    let mcc_risk = HashMap::from([("7802".to_string(), 0.75)]);

    let vector = vectorize(&request, &normalization(), &mcc_risk).unwrap();

    assert_eq!(vector[0], 1.0);
    assert_eq!(vector[1], 1.0);
    assert_eq!(vector[2], 1.0);
    assert_eq!(vector[5], 1.0);
    assert_eq!(vector[6], 1.0);
    assert_eq!(vector[7], 1.0);
    assert_eq!(vector[8], 1.0);
    assert_eq!(vector[9], 1.0);
    assert_eq!(vector[10], 0.0);
    assert_eq!(vector[11], 1.0);
    assert_eq!(vector[12], 0.75);
    assert_eq!(vector[13], 1.0);
}
