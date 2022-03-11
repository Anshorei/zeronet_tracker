use crate::peer_db::{PeerDB, PeerDatabase};
use std::time::SystemTime;

pub struct SharedState {
  pub peer_db:            Box<dyn PeerDatabase + Send>,
  pub start_time:         SystemTime,
}

impl SharedState {
  pub fn new() -> SharedState {
    SharedState {
      peer_db:            Box::new(PeerDB::new()),
      start_time:         SystemTime::now(),
    }
  }
}
