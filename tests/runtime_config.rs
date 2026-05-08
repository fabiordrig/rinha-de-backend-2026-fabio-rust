use std::{
    net::SocketAddr,
    sync::{Mutex, OnceLock},
};

use rinha_de_backend_2026_fabio_rust::config::server_address_from_env;

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn server_address_defaults_to_9999_when_port_is_missing() {
    let _guard = env_lock().lock().unwrap();
    let previous = std::env::var("PORT").ok();
    std::env::remove_var("PORT");

    let address = server_address_from_env().unwrap();

    assert_eq!(address, "0.0.0.0:9999".parse::<SocketAddr>().unwrap());
    restore_port(previous);
}

#[test]
fn server_address_uses_port_from_env() {
    let _guard = env_lock().lock().unwrap();
    let previous = std::env::var("PORT").ok();
    std::env::set_var("PORT", "10001");

    let address = server_address_from_env().unwrap();

    assert_eq!(address, "0.0.0.0:10001".parse::<SocketAddr>().unwrap());
    restore_port(previous);
}

fn restore_port(previous: Option<String>) {
    if let Some(value) = previous {
        std::env::set_var("PORT", value);
    } else {
        std::env::remove_var("PORT");
    }
}
