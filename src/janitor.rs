use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use log::*;

use crate::shared_state::SharedState;

pub fn run(shared_state: Arc<Mutex<SharedState>>, interval: u16, timeout: u16) {
  loop {
    sleep(Duration::from_secs(interval as u64));
    let mut shared_state = shared_state.lock().unwrap();

    let cutoff_timestamp = SystemTime::now() - Duration::from_secs(60 * timeout as u64);
    if cutoff_timestamp < shared_state.start_time {
      // Cutoff before start time of tracker. Wait with cleaning old peers
      // to give them time to announce again.
      continue;
    }

    let dead_peers = shared_state
      .peer_db
      .cleanup_peers(cutoff_timestamp)
      .unwrap();
    let stale_hashes = shared_state.peer_db.cleanup_hashes().unwrap();

    if dead_peers > 0 {
      info!("Removed {} dead peers", dead_peers);
    }

    if stale_hashes > 0 {
      info!("Removed {} stale hashes", stale_hashes);
    }
  }
}
