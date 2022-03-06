use std::time::SystemTime;

use zeronet_protocol::PeerAddr;

#[cfg(not(feature = "sql"))]
use crate::peer_db::basic::PeerDB;
#[cfg(feature = "sql")]
use crate::peer_db::sqlite::PeerDB;
use crate::peer_db::{Hash, Peer, PeerDatabase};

#[test]
fn test_update_peer() {
  use std::time::Duration;

  let mut peer_db = PeerDB::new().unwrap();
  let hashes = vec![Hash(vec![0u8])];
  let peer1 = Peer {
    address:    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    last_seen:  SystemTime::now(),
    date_added: SystemTime::now(),
  };
  let peer2 = Peer {
    date_added: SystemTime::now() + Duration::from_secs(5),
    ..peer1.clone()
  };

  assert_eq!(
    peer_db.update_peer(peer1, hashes.clone()),
    false,
    "Peer inserted"
  );
  assert_eq!(peer_db.update_peer(peer2, hashes), true, "Peer updated");
}

#[test]
fn test_remove_peer() {
  let mut peer_db = PeerDB::new().unwrap();
  let hashes = vec![Hash(vec![0u8])];
  let peer = Peer {
    address:    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    last_seen:  SystemTime::now(),
    date_added: SystemTime::now(),
  };

  peer_db.update_peer(peer.clone(), hashes);
  peer_db.remove_peer(&peer.address);
  assert_eq!(peer_db.get_peer_count(), 0);
}

#[test]
fn test_get_peer() {
  let mut peer_db = PeerDB::new().unwrap();
  let hash = Hash(vec![0u8]);
  let peer = Peer {
    address:    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    last_seen:  SystemTime::now(),
    date_added: SystemTime::now(),
  };
  peer_db.update_peer(peer, vec![hash]);

  assert_eq!(peer_db.get_peer_count(), 1);
}

#[test]
fn test_get_peer_count() {
  let mut peer_db = PeerDB::new().unwrap();
  let hash = Hash(vec![0u8]);
  let peer = Peer {
    address:    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    last_seen:  SystemTime::now(),
    date_added: SystemTime::now(),
  };

  assert_eq!(peer_db.get_peer_count(), 0);
  peer_db.update_peer(peer, vec![hash]);
  assert_eq!(peer_db.get_peer_count(), 1);
}

#[test]
fn test_get_hash_count() {
  let mut peer_db = PeerDB::new().unwrap();
  let hash = Hash(vec![0u8]);
  let peer = Peer {
    address:    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    last_seen:  SystemTime::now(),
    date_added: SystemTime::now(),
  };

  assert_eq!(peer_db.get_hash_count(), 0);
  peer_db.update_peer(peer, vec![hash]);
  assert_eq!(peer_db.get_hash_count(), 1);
}

#[test]
fn test_cleanup_peers() {
  let mut peer_db = PeerDB::new().unwrap();
  peer_db.cleanup_peers(SystemTime::now());
}

#[test]
fn test_cleanup_hashes() {
  let mut peer_db = PeerDB::new().unwrap();
  peer_db.cleanup_hashes();
}
