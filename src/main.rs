use std::{path::PathBuf, sync::Arc};

use axum::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rinha_de_backend_2026_fabio_rust::{
    app::build_app_with_engine,
    config::server_address_from_env,
    resources::load_resources_from_dir,
    scoring::ScoringEngine,
};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let resources_dir = std::env::var("RINHA_RESOURCES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("./resources"));

    let engine = match load_resources_from_dir(&resources_dir) {
        Ok(resources) => {
            tracing::info!(path = %resources_dir.display(), "loaded fraud resources");
            Arc::new(ScoringEngine::new(resources))
        }
        Err(error) => {
            tracing::error!(error = %error, path = %resources_dir.display(), "failed to load fraud resources, using fallback engine");
            Arc::new(ScoringEngine::empty())
        }
    };

    let address = server_address_from_env().unwrap();
    Server::bind(&address)
        .serve(build_app_with_engine(engine).into_make_service())
        .await
        .unwrap();
}
