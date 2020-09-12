use std::time::Instant;
use crate::peer_db::{PeerDB, PeerDatabase};

pub struct SharedState {
	pub peer_db: Box<dyn PeerDatabase + Send>,
	pub start_time:   Instant,
	pub opened_connections:  usize,
	pub closed_connections: usize,
	pub requests: usize,
}

impl SharedState {
	pub fn new() -> SharedState {
		SharedState {
			peer_db: Box::new(PeerDB::new()),
			start_time:   Instant::now(),
			opened_connections:  0,
			closed_connections: 0,
			requests: 0,
		}
	}
}
