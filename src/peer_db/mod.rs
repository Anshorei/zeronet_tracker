use std::time::Instant;
use zeronet_protocol::Address;
use std::collections::{HashMap, HashSet};

pub trait PeerDatabase {
  // Adds hashes for a peer, adding the peer if it is new
  fn update_peer(&mut self, peer: Peer, hashes: Vec<Vec<u8>>) -> Option<Peer>;
  // Remove a peer
  fn remove_peer(&mut self, peer: &Address) -> Option<Peer>;

  fn get_peer(&self, address: &Address) -> Option<&Peer>;
  fn get_peers(&self) -> Vec<Peer>;
  fn get_peers_for_hash(&self, hash: &Vec<u8>) -> Vec<Peer>;

  fn get_hashes(&self) -> Vec<(Vec<u8>, usize)>;

  fn get_peer_count(&self) -> usize;
  fn get_hash_count(&self) -> usize;

  // Remove peers that have not announced since
  // the given timestamp
  fn cleanup_peers(&mut self, timestamp: Instant) -> usize;
  // Remove peerless hashes
  fn cleanup_hashes(&mut self) -> usize;
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

pub struct PeerDB {
  pub peers: HashMap<Address, Peer>,
  pub hashes: HashMap<Vec<u8>, Hash>,
  hash_to_peer: HashMap<Vec<u8>, HashSet<Address>>,
  peer_to_hash: HashMap<Address, HashSet<Vec<u8>>>,
}

impl PeerDB {
  pub fn new() -> PeerDB {
    PeerDB {
      peers: HashMap::new(),
      hashes: HashMap::new(),
      hash_to_peer: HashMap::new(),
      peer_to_hash: HashMap::new(),
    }
  }
  fn add_hash(&mut self, hash: &Vec<u8>) {
    if self.hashes.contains_key(hash) {
      return
    }
    let new_hash = Hash {
      hash: hash.clone(),
      date_added: Instant::now(),
    };
    self.hashes.insert(hash.clone(), new_hash);
    self.hash_to_peer.insert(hash.clone(), HashSet::new());
  }
  fn link(&mut self, hash: &Vec<u8>, peer: &Address) -> Option<()> {
    self.hash_to_peer
      .get_mut(hash)?
      .insert(peer.clone());
    self.peer_to_hash
      .get_mut(peer)?
      .insert(hash.clone());
    Option::None
  }
}

impl PeerDatabase for PeerDB {
  fn update_peer(&mut self, peer: Peer, hashes: Vec<Vec<u8>>) -> Option<Peer> {
    if !self.peer_to_hash.contains_key(&&peer.address) {
      self.peer_to_hash.insert(peer.address.clone(), HashSet::new());
    }
    for hash in hashes.iter() {
      self.add_hash(hash);
      self.link(hash, &&peer.address);
    }
    self.peers.insert(peer.address.clone(), peer)
  }

  fn remove_peer(&mut self, address: &Address) -> Option<Peer> {
    let hashes = self.peer_to_hash.remove(address).unwrap_or(HashSet::new());
    for hash in hashes.iter() {
      self.hash_to_peer
        .get_mut(hash)
        .unwrap()
        .remove(address);
    }
    self.peers.remove(address)
  }

  fn get_peer(&self, address: &Address) -> Option<&Peer> {
    self.peers.get(address)
  }

  fn get_peers(&self) -> Vec<Peer> {
    self.peers.values()
      .map(|peer| peer.clone())
      .collect()
  }

  fn get_peers_for_hash(&self, hash: &Vec<u8>) -> Vec<Peer> {
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

  fn get_hashes(&self) -> Vec<(Vec<u8>, usize)> {
    self.hash_to_peer.iter()
      .map(|(hash, set)| (hash.clone(), set.len()))
      .collect()
  }

  fn get_peer_count(&self) -> usize {
    self.peers.len()
  }

  fn get_hash_count(&self) -> usize {
    self.hashes.len()
  }

  fn cleanup_peers(&mut self, timestamp: Instant) -> usize {
    let dead_peers: Vec<_> = self
      .peers
      .values()
      .filter(|peer| peer.last_seen < timestamp)
      .map(|peer| peer.address.clone())
      .collect();
    let count = dead_peers.len();
    dead_peers.iter().for_each(|peer| { self.remove_peer(&peer); });
    count
  }

  fn cleanup_hashes(&mut self) -> usize {
    let dead_hashes: Vec<_> = self
      .hashes
      .values()
      .filter(|hash| self
        .hash_to_peer
        .get(&hash.hash)
        .map_or(0, |h| h.len()) == 0)
      .map(|hash| hash.hash.clone())
      .collect();
    let count = dead_hashes.len();
    for hash in dead_hashes.iter() {
      self.hashes.remove(hash);
    }
    count
  }
}
