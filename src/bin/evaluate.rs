use std::path::PathBuf;

use clap::Parser;
use rinha_de_backend_2026_fabio_rust::{
    resources::load_resources_from_dir,
    scoring::{evaluate_entries, load_evaluation_entries_from_file, ScoringEngine},
};

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "./resources")]
    resources_dir: PathBuf,
    #[arg(long)]
    dataset_path: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let resources = load_resources_from_dir(&args.resources_dir)?;
    let engine = ScoringEngine::new(resources);
    let entries = load_evaluation_entries_from_file(&args.dataset_path)?;
    let summary = evaluate_entries(&engine, &entries)?;

    println!("total={}", summary.total);
    println!("correct={}", summary.correct);
    println!("accuracy={:.6}", summary.accuracy);
    println!("precision_fraud={:.6}", summary.precision_fraud);
    println!("recall_fraud={:.6}", summary.recall_fraud);
    println!("true_positive={}", summary.true_positive);
    println!("true_negative={}", summary.true_negative);
    println!("false_positive={}", summary.false_positive);
    println!("false_negative={}", summary.false_negative);
    println!("avg_score_error={:.6}", summary.avg_score_error);

    Ok(())
}
