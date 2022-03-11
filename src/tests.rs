use std::sync::{Arc, Mutex};

use futures::executor::block_on;
use zeronet_protocol::{PeerAddr, ZeroConnection};

use crate::shared_state::SharedState;
use crate::start_listener;

fn start_tracker() {
  std::env::set_var("RUST_LOG", "zeronet_tracker=trace");

  let shared_state = Arc::new(Mutex::new(SharedState::new()));
  start_listener(&shared_state, "localhost".to_string(), 15442);
}

fn handshake() -> serde_json::Value {
  let text = r#"
    {
      "crypt": null,
      "crypt_supported": ["tls-rsa"],
      "fileserver_port": 15441,
      "onion": "zp2ynpztyxj2kw7x",
      "protocol": "v2",
      "port_opened": true,
      "peer_id": "-ZN0056-DMK3XX30mOrw",
      "rev": 2122,
      "target_ip": "192.168.1.13",
      "version": "0.5.6"
    }"#;
  let value = serde_json::from_str(text).unwrap();
  value
}

fn announce() -> serde_json::Value {
  let text = r#"
    {
      "hashes": [],
      "onions": [],
      "onion_signs": [],
      "onion_sign_this": "",
      "port": 15441,
      "need_types": ["ipv4"],
      "need_num": 20,
      "add": ["onion"]
    }"#;
  let value = serde_json::from_str(text).unwrap();
  value
}

#[test]
#[cfg(not(feature = "tor"))]
fn test_handshake_with_onion() {
  start_tracker();

  let address = PeerAddr::parse("127.0.0.1:15442".to_string()).unwrap();
  let mut conn = ZeroConnection::from_address(address).unwrap();
  let handshake_future = conn.request("handshake", handshake());
  let response = block_on(handshake_future).unwrap();
  assert_eq!(response.to, conn.last_req_id());

  let announce_future = conn.request("announce", announce());
  let response = block_on(announce_future).unwrap();
  assert_eq!(response.to, conn.last_req_id());
}
