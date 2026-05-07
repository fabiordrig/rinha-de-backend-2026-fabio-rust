use axum::Server;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rinha_de_backend_2026_fabio_rust::app::build_app;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let address = "0.0.0.0:9999".parse().unwrap();
    Server::bind(&address)
        .serve(build_app().into_make_service())
        .await
        .unwrap();
}
