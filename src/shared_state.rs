use std::time::SystemTime;

use zeronet_peerdb::{Error, PeerDatabase, PeerDB};

use crate::args::Args;

pub struct SharedState {
  pub peer_db:    Box<dyn PeerDatabase<Error = Error> + Send>,
  pub start_time: SystemTime,
}

impl SharedState {
  pub fn new(args: &Args) -> SharedState {

    SharedState {
      #[cfg(feature = "sql")]
      peer_db:    Box::new(PeerDB::new(args.database_file.clone()).unwrap()),
      #[cfg(not(feature = "sql"))]
      peer_db:    Box::new(PeerDB::new().unwrap()),
      start_time: SystemTime::now(),
    }
  }
}
