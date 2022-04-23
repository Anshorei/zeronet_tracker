use std::time::SystemTime;

use zeronet_protocol::PeerAddr;

use crate::args::get_arguments;
#[cfg(not(feature = "sql"))]
use crate::peer_db::basic::PeerDB;
#[cfg(feature = "sql")]
use crate::peer_db::sqlite::PeerDB;
use crate::peer_db::{Hash, Peer, PeerDatabase};

#[test]
fn test_update_peer() {
  use std::time::Duration;

  let args = get_arguments();
  let mut peer_db = PeerDB::new(&args).unwrap();
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
    peer_db.update_peer(peer1, hashes.clone()).unwrap(),
    false,
    "Peer inserted"
  );
  assert_eq!(
    peer_db.update_peer(peer2, hashes).unwrap(),
    true,
    "Peer updated"
  );
}

#[test]
fn test_remove_peer() {
  let args = get_arguments();
  let mut peer_db = PeerDB::new(&args).unwrap();
  let hashes = vec![Hash(vec![0u8])];
  let peer = Peer {
    address:    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    last_seen:  SystemTime::now(),
    date_added: SystemTime::now(),
  };

  peer_db
    .update_peer(peer.clone(), hashes)
    .expect("Could not update peer");
  assert!(peer_db.remove_peer(&peer.address).is_ok());
  assert_eq!(peer_db.get_peer_count().unwrap(), 0);
}

#[test]
fn test_get_peer() {
  let args = get_arguments();
  let mut peer_db = PeerDB::new(&args).unwrap();
  let hash = Hash(vec![0u8]);
  let peer = Peer {
    address:    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    last_seen:  SystemTime::now(),
    date_added: SystemTime::now(),
  };
  peer_db
    .update_peer(peer, vec![hash])
    .expect("Could not update peer");

  assert_eq!(peer_db.get_peer_count().unwrap(), 1);
}

#[test]
fn test_get_peer_count() {
  let args = get_arguments();
  let mut peer_db = PeerDB::new(&args).unwrap();
  let hash = Hash(vec![0u8]);
  let peer = Peer {
    address:    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    last_seen:  SystemTime::now(),
    date_added: SystemTime::now(),
  };

  assert_eq!(peer_db.get_peer_count().unwrap(), 0);
  peer_db
    .update_peer(peer, vec![hash])
    .expect("Could not update peer");
  assert_eq!(peer_db.get_peer_count().unwrap(), 1);
}

#[test]
fn test_get_hash_count() {
  let args = get_arguments();
  let mut peer_db = PeerDB::new(&args).unwrap();
  let hash = Hash(vec![0u8]);
  let peer = Peer {
    address:    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    last_seen:  SystemTime::now(),
    date_added: SystemTime::now(),
  };

  assert_eq!(peer_db.get_hash_count().unwrap(), 0);
  peer_db
    .update_peer(peer, vec![hash])
    .expect("Could not update peer");
  assert_eq!(peer_db.get_hash_count().unwrap(), 1);
}

#[test]
fn test_get_peers_for_hash() {
  let args = get_arguments();
  let mut peer_db = PeerDB::new(&args).unwrap();
  let hash = Hash(vec![0u8]);
  let peer = Peer {
    address:    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    last_seen:  SystemTime::now(),
    date_added: SystemTime::now(),
  };
  peer_db
    .update_peer(peer, vec![hash.clone()])
    .expect("Could not update peer");

  let peers = peer_db
    .get_peers_for_hash(&hash)
    .expect("Could not get peers for hash");

  assert_eq!(peers.len(), 1);
  assert_eq!(
    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    peers[0].address
  );
}

#[test]
fn test_get_hashes() {
  let args = get_arguments();
  let mut peer_db = PeerDB::new(&args).unwrap();
  let hash = Hash(vec![0u8]);
  let peer = Peer {
    address:    PeerAddr::parse("127.0.0.1:11111").unwrap(),
    last_seen:  SystemTime::now(),
    date_added: SystemTime::now(),
  };
  peer_db
    .update_peer(peer, vec![hash])
    .expect("Could not update peer");

  let hashes = peer_db.get_hashes().expect("Could not get hashes");

  assert_eq!(hashes.len(), 1);
  let (hash, peercount) = &hashes[0];
  assert_eq!(&Hash(vec![0u8]), hash);
  assert_eq!(&1, peercount);
}

#[test]
fn test_cleanup_peers() {
  let args = get_arguments();
  let mut peer_db = PeerDB::new(&args).unwrap();
  let result = peer_db.cleanup_peers(SystemTime::now());
  assert!(result.is_ok());
}

#[test]
fn test_cleanup_hashes() {
  let args = get_arguments();
  let mut peer_db = PeerDB::new(&args).unwrap();
  let result = peer_db.cleanup_hashes();
  assert!(result.is_ok());
}
