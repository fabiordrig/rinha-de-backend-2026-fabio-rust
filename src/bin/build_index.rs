use std::path::PathBuf;

use clap::Parser;
use rinha_de_backend_2026_fabio_rust::index::build_index_from_resources_dir;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long)]
    resources_dir: PathBuf,
    #[arg(long)]
    output_path: PathBuf,
}

fn main() {
    let args = Args::parse();

    build_index_from_resources_dir(&args.resources_dir, &args.output_path)
        .unwrap_or_else(|error| panic!("failed to build index: {error}"));

    println!("index_written={}", args.output_path.display());
}
