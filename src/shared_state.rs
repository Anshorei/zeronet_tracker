use log::*;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use zeronet_protocol::Address;

pub struct SharedState {
	pub peers: HashMap<Address, Peer>,
	pub hashes: HashMap<Vec<u8>, Hash>,
	pub hash_to_peer: HashMap<Vec<u8>, HashSet<Address>>,
	pub start_time: Instant,
	pub connections: usize,
}

impl SharedState {
	pub fn new() -> SharedState {
		SharedState {
			peers: HashMap::new(),
			hashes: HashMap::new(),
			hash_to_peer: HashMap::new(),
			start_time: Instant::now(),
			connections: 0,
		}
	}

	pub fn insert_peer(&mut self, peer: Peer, hashes: Vec<Vec<u8>>) {
		for hash in hashes.iter() {
			if !self.hashes.contains_key(hash) {
				let new_hash = Hash {
					hash: hash.clone(),
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

		let peer_address = peer.address.to_string();
		let result = self.peers.insert(peer.address.clone(), peer);
		match result {
			Some(_) => trace!("Updated peer {} for {} hashes", peer_address, hashes.len()),
			None => trace!("Added peer {} for {} hashes", peer_address, hashes.len()),
		}
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
	pub address: Address,
	pub date_added: Instant,
	pub last_seen: Instant,
}

#[derive(Clone)]
pub struct Hash {
	pub hash: Vec<u8>,
	pub date_added: Instant,
}
