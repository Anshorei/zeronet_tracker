use log::*;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use zeronet_protocol::Address;
use crate::peer_db::{Peer, Hash, PeerDB, PeerDatabase};

pub struct SharedState {
	pub peer_db: Box<dyn PeerDatabase + Send>,
	pub start_time:   Instant,
	pub connections:  usize,
}

impl SharedState {
	pub fn new() -> SharedState {
		SharedState {
			peer_db: Box::new(PeerDB::new()),
			start_time:   Instant::now(),
			connections:  0,
		}
	}

	pub fn insert_peer(&mut self, peer: Peer, hashes: Vec<Vec<u8>>) -> Option<Peer> {
		for hash in hashes.iter() {
			if !self.hashes.contains_key(hash) {
				let new_hash = Hash {
					hash:       hash.clone(),
					date_added: Instant::now(),
				};
				self.hashes.insert(hash.clone(), new_hash);
			}
			if !self.hash_to_peer.contains_key(hash) {
				self.hash_to_peer.insert(hash.clone(), HashSet::new());
			}
			self
				.hash_to_peer
				.get_mut(hash)
				.unwrap()
				.insert(peer.address.clone());
		}

		self.peers.insert(peer.address.clone(), peer)
	}

	pub fn get_peers(&self, hash: &Vec<u8>) -> Vec<Peer> {
		let peers = self.hash_to_peer.get(hash);
		let peers = match peers {
			Some(peers) => peers.iter().collect(),
			None => vec![],
		};

		peers
			.iter()
			.map(|peer_id| {
				let peer = self.peers.get(*peer_id).unwrap();
				peer.clone()
			})
			.collect()
	}

	pub fn peer_count(&self) -> usize {
		self.peers.len()
	}

	pub fn hash_count(&self) -> usize {
		self.hashes.len()
	}
}

#[derive(Clone)]
pub struct Peer {
	pub address:    Address,
	pub date_added: Instant,
	pub last_seen:  Instant,
}

#[derive(Clone)]
pub struct Hash {
	pub hash:       Vec<u8>,
	pub date_added: Instant,
}
