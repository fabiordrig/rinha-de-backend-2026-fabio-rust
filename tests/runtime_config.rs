use std::net::SocketAddr;

use rinha_de_backend_2026_fabio_rust::config::server_address_from_env;

#[test]
fn server_address_defaults_to_9999_when_port_is_missing() {
    std::env::remove_var("PORT");

    let address = server_address_from_env().unwrap();

    assert_eq!(address, "0.0.0.0:9999".parse::<SocketAddr>().unwrap());
}

#[test]
fn server_address_uses_port_from_env() {
    std::env::set_var("PORT", "10001");

    let address = server_address_from_env().unwrap();

    assert_eq!(address, "0.0.0.0:10001".parse::<SocketAddr>().unwrap());
    std::env::remove_var("PORT");
}
