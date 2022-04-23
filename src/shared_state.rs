use std::time::SystemTime;

use crate::args::Args;
#[cfg(not(feature = "sql"))]
use crate::peer_db::basic::{Error, PeerDB};
#[cfg(feature = "sql")]
use crate::peer_db::sqlite::{Error, PeerDB};
use crate::peer_db::PeerDatabase;

pub struct SharedState {
  pub peer_db:    Box<dyn PeerDatabase<Error = Error> + Send>,
  pub start_time: SystemTime,
}

impl SharedState {
  pub fn new(args: &Args) -> SharedState {
    SharedState {
      peer_db:    Box::new(PeerDB::new(&args).unwrap()),
      start_time: SystemTime::now(),
    }
  }
}
