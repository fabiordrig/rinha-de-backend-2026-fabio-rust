use std::{fs, io::Write, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

use flate2::{write::GzEncoder, Compression};

use rinha_de_backend_2026_fabio_rust::{
    index::{build_index_from_resources_dir, load_index_file},
    resources::load_resources_from_dir,
    types::ReferenceLabel,
};

fn temp_fixture_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    let dir = std::env::temp_dir().join(format!("rinha-binary-index-{unique}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_gzip(path: &PathBuf, content: &str) {
    let file = fs::File::create(path).unwrap();
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(content.as_bytes()).unwrap();
    encoder.finish().unwrap();
}

#[test]
fn builds_and_loads_binary_index_roundtrip() {
    let fixture_dir = temp_fixture_dir();
    let output_path = fixture_dir.join("references.bin");

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
          { "vector": [0.9506, 0.8333, 1.0, 0.2174, 0.8333, -1.0, -1.0, 0.9523, 1.0, 0.0, 1.0, 1.0, 0.75, 0.0055], "label": "fraud" }
        ]"#,
    );

    build_index_from_resources_dir(&fixture_dir, &output_path).unwrap();
    let index = load_index_file(&output_path).unwrap();

    assert_eq!(index.len(), 2);
    assert_eq!(index[0].label, ReferenceLabel::Legit);
    assert_eq!(index[1].label, ReferenceLabel::Fraud);
    assert_eq!(index[0].vector[0], 0.0385);
    assert_eq!(index[1].vector[5], -1.0);
}

#[test]
fn loads_references_from_binary_index_when_available() {
    let fixture_dir = temp_fixture_dir();
    let output_path = fixture_dir.join("references.bin");

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
          { "vector": [0.9506, 0.8333, 1.0, 0.2174, 0.8333, -1.0, -1.0, 0.9523, 1.0, 0.0, 1.0, 1.0, 0.75, 0.0055], "label": "fraud" }
        ]"#,
    );

    build_index_from_resources_dir(&fixture_dir, &output_path).unwrap();
    fs::remove_file(fixture_dir.join("references.json.gz")).unwrap();

    let resources = load_resources_from_dir(&fixture_dir).unwrap();

    assert_eq!(resources.references.len(), 2);
    assert_eq!(resources.references[0].label, ReferenceLabel::Legit);
    assert_eq!(resources.references[1].label, ReferenceLabel::Fraud);
}
