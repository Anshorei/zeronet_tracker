use log::*;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use zeronet_protocol::Address;
use crate::peer_db::{Peer, Hash, PeerDB, PeerDatabase};

pub struct SharedState {
	pub peer_db: Box<dyn PeerDatabase + Send>,
	pub start_time:   Instant,
	pub connections:  usize,
	pub requests: usize,
}

impl SharedState {
	pub fn new() -> SharedState {
		SharedState {
			peer_db: Box::new(PeerDB::new()),
			start_time:   Instant::now(),
			connections:  0,
			requests: 0,
		}
	}
}
