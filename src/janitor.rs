use crate::shared_state::SharedState;
use log::*;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, SystemTime};

pub fn run(shared_state: Arc<Mutex<SharedState>>, interval: u16, timeout: u16) {
  loop {
    sleep(Duration::from_secs(interval as u64));
    let mut shared_state = shared_state.lock().unwrap();
    let cutoff = SystemTime::now() - Duration::from_secs(60 * timeout as u64);
    let dead_peers = shared_state.peer_db.cleanup_peers(cutoff);
    let stale_hashes = shared_state.peer_db.cleanup_hashes();
    if dead_peers > 0 {
      info!("Removed {} dead peers", dead_peers);
    }
    if stale_hashes > 0 {
      info!("Removed {} stale hashes", stale_hashes);
    }
  }
}
