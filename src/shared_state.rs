use std::time::SystemTime;

#[cfg(not(feature = "sqlite"))]
use crate::peer_db::basic::PeerDB;
#[cfg(feature = "sqlite")]
use crate::peer_db::sqlite::PeerDB;
use crate::peer_db::PeerDatabase;

pub struct SharedState {
  pub peer_db:    Box<dyn PeerDatabase + Send>,
  pub start_time: SystemTime,
}

impl SharedState {
  pub fn new() -> SharedState {
    SharedState {
      peer_db:    Box::new(PeerDB::new().unwrap()),
      start_time: SystemTime::now(),
    }
  }
}
