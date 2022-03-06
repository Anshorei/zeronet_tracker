use lazy_static::lazy_static;
use prometheus::{IntCounter, IntGauge, register_int_counter, register_int_gauge};

lazy_static! {
  pub static ref PEER_GAUGE: IntGauge = register_int_gauge!("peers", "Peers in database").unwrap();
  pub static ref HASH_GAUGE: IntGauge =
    register_int_gauge!("hashes", "Hashes in database").unwrap();
  pub static ref REQUEST_COUNTER: IntCounter =
    register_int_counter!("requests", "Requests received").unwrap();
  pub static ref OPENED_CONNECTIONS: IntCounter =
    register_int_counter!("opened_connections", "Connections opened since start").unwrap();
  pub static ref CLOSED_CONNECTIONS: IntCounter =
    register_int_counter!("closed_connections", "Connections closed since start").unwrap();
}
