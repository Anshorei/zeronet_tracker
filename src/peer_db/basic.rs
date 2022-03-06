use std::collections::{HashMap, HashSet};
use std::time::SystemTime;

use zeronet_protocol::PeerAddr as Address;

use super::{Hash, Peer, PeerDatabase};

#[derive(Clone)]
pub struct StoredHash {
  pub hash:       Hash,
  pub date_added: SystemTime,
}

pub struct PeerDB {
  pub peers:    HashMap<Address, Peer>,
  pub hashes:   HashMap<Hash, StoredHash>,
  hash_to_peer: HashMap<Hash, HashSet<Address>>,
  peer_to_hash: HashMap<Address, HashSet<Hash>>,
}

impl PeerDB {
  pub fn new() -> Result<PeerDB, ()> {
    let db = PeerDB {
      peers:        HashMap::new(),
      hashes:       HashMap::new(),
      hash_to_peer: HashMap::new(),
      peer_to_hash: HashMap::new(),
    };
    Ok(db)
  }

  pub fn insert_hash(&mut self, hash: &Hash) {
    if self.hashes.contains_key(hash) {
      return;
    }
    let new_hash = StoredHash {
      hash:       hash.clone(),
      date_added: SystemTime::now(),
    };
    self.hashes.insert(hash.clone(), new_hash);
    self.hash_to_peer.insert(hash.clone(), HashSet::new());
  }

  fn link(&mut self, hash: &Hash, peer: &Address) -> Option<()> {
    self.hash_to_peer.get_mut(hash)?.insert(peer.clone());
    self.peer_to_hash.get_mut(peer)?.insert(hash.clone());
    Option::None
  }
}

impl PeerDatabase for PeerDB {
  fn update_peer(&mut self, peer: Peer, hashes: Vec<Hash>) -> bool {
    if !self.peer_to_hash.contains_key(&&peer.address) {
      self
        .peer_to_hash
        .insert(peer.address.clone(), HashSet::new());
    }
    for hash in hashes.iter() {
      self.insert_hash(hash);
      self.link(hash, &&peer.address);
    }
    match self.peers.insert(peer.address.clone(), peer) {
      None => false,
      Some(_) => true,
    }
  }

  fn remove_peer(&mut self, peer_address: &Address) -> Option<Peer> {
    let hashes = self
      .peer_to_hash
      .remove(peer_address)
      .unwrap_or(HashSet::new());

    for hash in hashes.iter() {
      self
        .hash_to_peer
        .get_mut(hash)
        .unwrap()
        .remove(peer_address);
    }

    self.peers.remove(peer_address)
  }

  fn get_peer(&self, peer_address: &Address) -> Option<Peer> {
    self.peers.get(peer_address).map(|peer| peer.clone())
  }

  fn get_peers(&self) -> Vec<Peer> {
    self.peers.values().map(|peer| peer.clone()).collect()
  }

  fn get_peers_for_hash(&self, hash: &Hash) -> Vec<Peer> {
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

  fn get_hashes(&self) -> Vec<(Hash, usize)> {
    self
      .hash_to_peer
      .iter()
      .map(|(hash, set)| (hash.clone(), set.len()))
      .collect()
  }

  fn get_peer_count(&self) -> usize {
    self.peers.len()
  }

  fn get_hash_count(&self) -> usize {
    self.hashes.len()
  }

  fn cleanup_peers(&mut self, timestamp: SystemTime) -> usize {
    let dead_peers: Vec<_> = self
      .peers
      .values()
      .filter(|peer| peer.last_seen < timestamp)
      .map(|peer| peer.address.clone())
      .collect();
    let count = dead_peers.len();
    dead_peers.iter().for_each(|peer| {
      self.remove_peer(&peer);
    });

    count
  }

  fn cleanup_hashes(&mut self) -> usize {
    let dead_hashes: Vec<_> = self
      .hashes
      .values()
      .filter(|hash| self.hash_to_peer.get(&hash.hash).map_or(0, |h| h.len()) == 0)
      .map(|hash| hash.hash.clone())
      .collect();
    let count = dead_hashes.len();
    for hash in dead_hashes.iter() {
      self.hashes.remove(hash);
    }

    count
  }
}
