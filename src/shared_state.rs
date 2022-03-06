use crate::peer_db::{PeerDB, PeerDatabase};
use std::time::SystemTime;

pub struct SharedState {
  pub peer_db:            Box<dyn PeerDatabase + Send>,
  pub start_time:         SystemTime,
  pub opened_connections: usize,
  pub closed_connections: usize,
  pub requests:           usize,
}

impl SharedState {
  pub fn new() -> SharedState {
    SharedState {
      peer_db:            Box::new(PeerDB::new()),
      start_time:         SystemTime::now(),
      opened_connections: 0,
      closed_connections: 0,
      requests:           0,
    }
  }
}
