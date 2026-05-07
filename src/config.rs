use std::{net::SocketAddr, num::ParseIntError};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid PORT value: {0}")]
    InvalidPort(#[from] ParseIntError),
}

pub fn server_address_from_env() -> Result<SocketAddr, ConfigError> {
    let port: u16 = std::env::var("PORT")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.parse())
        .transpose()?
        .unwrap_or(9999);

    Ok(([0, 0, 0, 0], port).into())
}
