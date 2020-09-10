use std::time::{Duration, Instant};
use std::thread::sleep;
use crate::peer_db::{PeerDatabase};
use crate::shared_state::SharedState;
use std::sync::{Arc, Mutex};
use log::*;

pub fn run(shared_state: Arc<Mutex<SharedState>>) {
  loop {
    sleep(Duration::from_secs(60 * 5));
    let mut shared_state = shared_state.lock().unwrap();
    let cutoff = Instant::now() - Duration::from_secs(60 * 40);
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
