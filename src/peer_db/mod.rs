use std::time::SystemTime;

use zeronet_protocol::PeerAddr as Address;

#[cfg(not(feature = "sql"))]
pub mod basic;
#[cfg(feature = "sql")]
pub mod sqlite;

#[cfg(test)]
mod tests;

pub fn get_peer_db_type() -> &'static str {
  #[cfg(feature = "sql")]
  return "SQLite";
  #[cfg(not(feature = "sql"))]
  return "Basic";
}

#[derive(Clone)]
pub struct Peer {
  pub address:    Address,
  pub date_added: SystemTime,
  pub last_seen:  SystemTime,
}

#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hash(pub Vec<u8>);

pub trait PeerDatabase {
  // Adds hashes for a peer, adding the peer if it is new.
  // Returns true if the peer was update, and false if it was inserted.
  fn update_peer(&mut self, peer: Peer, hashes: Vec<Hash>) -> bool;
  // Remove a peer, returning it if it exists
  fn remove_peer(&mut self, peer_address: &Address) -> Option<Peer>;

  fn get_peer(&self, peer_address: &Address) -> Option<Peer>;
  fn get_peers(&self) -> Vec<Peer>;
  fn get_peers_for_hash(&self, hash: &Hash) -> Vec<Peer>;

  fn get_hashes(&self) -> Vec<(Hash, usize)>;

  fn get_peer_count(&self) -> usize;
  fn get_hash_count(&self) -> usize;

  // Remove peers that have not announced since
  // the given timestamp
  fn cleanup_peers(&mut self, timestamp: SystemTime) -> usize;
  // Remove peerless hashes
  fn cleanup_hashes(&mut self) -> usize;
}
