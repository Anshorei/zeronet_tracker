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
  type Error;

  // Adds hashes for a peer, adding the peer if it is new.
  // Returns true if the peer was update, and false if it was inserted.
  fn update_peer(&mut self, peer: Peer, hashes: Vec<Hash>) -> Result<bool, Self::Error>;
  // Remove a peer, returning it if it exists
  fn remove_peer(&mut self, peer_address: &Address) -> Result<Option<Peer>, Self::Error>;

  fn get_peer(&self, peer_address: &Address) -> Result<Option<Peer>, Self::Error>;
  fn get_peers(&self) -> Result<Vec<Peer>, Self::Error>;
  fn get_peers_for_hash(&self, hash: &Hash) -> Result<Vec<Peer>, Self::Error>;

  fn get_hashes(&self) -> Result<Vec<(Hash, usize)>, Self::Error>;

  fn get_peer_count(&self) -> Result<usize, Self::Error>;
  fn get_hash_count(&self) -> Result<usize, Self::Error>;

  // Remove peers that have not announced since
  // the given timestamp
  fn cleanup_peers(&mut self, timestamp: SystemTime) -> Result<usize, Self::Error>;
  // Remove peerless hashes
  fn cleanup_hashes(&mut self) -> Result<usize, Self::Error>;
}
