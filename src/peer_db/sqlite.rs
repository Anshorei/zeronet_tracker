use std::time::{Duration, SystemTime, UNIX_EPOCH};

use sqlite::{Connection, Value};
use zeronet_protocol::PeerAddr;

use super::{Hash, Peer, PeerDatabase};

fn unix_to_timestamp(seconds: i64) -> SystemTime {
  UNIX_EPOCH
    .checked_add(Duration::from_secs(seconds as u64))
    .unwrap()
}

fn timestamp_to_unix(timestamp: SystemTime) -> i64 {
  timestamp.duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

pub struct PeerDB {
  conn: Connection,
}

impl PeerDB {
  pub fn new() -> Result<PeerDB, ()> {
    let connection = sqlite::open(":memory:").unwrap();
    connection
      .execute(
        "
      CREATE TABLE peers (
        pk INTEGER PRIMARY KEY AUTOINCREMENT,
        address TEXT UNIQUE NOT NULL,
        date_added TIMESTAMP,
        last_seen TIMESTAMP
      );
      CREATE TABLE hashes (
        pk INTEGER PRIMARY KEY AUTOINCREMENT,
        hash BLOB UNIQUE NOT NULL
      );
      CREATE TABLE peer_hashes (
        peer_pk INTEGER REFERENCES peers(pk),
        hash_pk INTEGER REFERENCES hashes(pk),
        UNIQUE(peer_pk, hash_pk)
      );
    ",
      )
      .unwrap();
    let db = PeerDB { conn: connection };
    return Ok(db);
  }

  pub fn upsert_peer(&mut self, peer: &Peer) -> bool {
    let mut statement = self
      .conn
      .prepare(
        "
      INSERT INTO peers
        (address, date_added, last_seen)
      VALUES
        (:address, :date_added, :last_seen)
      ON CONFLICT (address) DO UPDATE SET
        last_seen = :last_seen
      RETURNING last_seen;
      ",
      )
      .unwrap();
    let date_updated = timestamp_to_unix(peer.date_added);
    statement
      .bind_by_name(":address", peer.address.to_string().as_str())
      .unwrap();
    statement.bind_by_name(":date_added", date_updated).unwrap();
    statement
      .bind_by_name(":last_seen", timestamp_to_unix(peer.last_seen))
      .unwrap();
    statement.next().unwrap();
    let date_added = statement.read::<i64>(0).unwrap();
    println!("{}, {}", date_updated, date_added);
    return date_updated != date_added;
  }

  pub fn insert_hash(&mut self, hash: &Hash) {
    let mut statement = self
      .conn
      .prepare(
        "
      INSERT INTO hashes
        (hash)
      VALUES
        (:hash)
      ON CONFLICT (hash) DO NOTHING;
    ",
      )
      .unwrap();
    statement.bind_by_name(":hash", hash.0.as_slice()).unwrap();
    statement.next().unwrap();
  }

  pub fn link(&mut self, hash: &Hash, peer_address: &PeerAddr) {
    let mut statement = self
      .conn
      .prepare(
        "
      INSERT INTO peer_hashes
        (peer_pk, hash_pk)
      VALUES (
        (SELECT pk FROM peers WHERE address = ?),
        (SELECT pk FROM hashes WHERE hash = ?)
      )
      ON CONFLICT (peer_pk, hash_pk) DO NOTHING;
    ",
      )
      .unwrap();
    statement
      .bind(1, peer_address.to_string().as_str())
      .unwrap();
    statement.bind(2, hash.0.as_slice()).unwrap();
    statement.next().unwrap();
  }
}

impl PeerDatabase for PeerDB {
  fn update_peer(&mut self, peer: Peer, hashes: Vec<Hash>) -> bool {
    let was_known_peer = self.upsert_peer(&peer);
    for hash in hashes.iter() {
      self.insert_hash(&hash);
      self.link(&hash, &peer.address);
    }
    return was_known_peer;
  }

  fn remove_peer(&mut self, peer_address: &PeerAddr) -> Option<Peer> {
    let mut statement = self
      .conn
      .prepare(
        "
      DELETE FROM peer_hashes
      WHERE peer_pk IN (
        SELECT pk FROM peers WHERE address = ?
      );
    ",
      )
      .unwrap();
    statement
      .bind(1, peer_address.to_string().as_str())
      .unwrap();
    let mut cursor = self
      .conn
      .prepare(
        "
      DELETE FROM peers
      WHERE address = ?
      RETURNING address, date_added, last_seen;
    ",
      )
      .unwrap()
      .into_cursor();
    cursor
      .bind(&[Value::String(peer_address.to_string())])
      .unwrap();
    if let Some(row) = cursor.next().unwrap() {
      return Some(Peer {
        address:    PeerAddr::parse(row[0].as_string().unwrap()).unwrap(),
        date_added: unix_to_timestamp(row[1].as_integer().unwrap()),
        last_seen:  unix_to_timestamp(row[2].as_integer().unwrap()),
      });
    } else {
      return None;
    }
  }

  fn get_peer(&self, peer_address: &PeerAddr) -> Option<Peer> {
    let mut cursor = self
      .conn
      .prepare(
        "
      SELECT address, date_added, last_seen
      FROM peers
      WHERE address = ?;
    ",
      )
      .unwrap()
      .into_cursor();
    cursor
      .bind(&[Value::String(peer_address.to_string())])
      .unwrap();
    if let Some(row) = cursor.next().unwrap() {
      return Some(Peer {
        address:    PeerAddr::parse(row[0].as_string().unwrap()).unwrap(),
        date_added: unix_to_timestamp(row[1].as_integer().unwrap()),
        last_seen:  unix_to_timestamp(row[2].as_integer().unwrap()),
      });
    } else {
      return None;
    }
  }

  fn get_peers(&self) -> Vec<Peer> {
    let mut cursor = self
      .conn
      .prepare(
        "
      SELECT address, date_added, last_seen
      FROM peers;
    ",
      )
      .unwrap()
      .into_cursor();
    let mut peers = Vec::new();
    while let Some(row) = cursor.next().unwrap() {
      peers.push(Peer {
        address:    PeerAddr::parse(row[0].as_string().unwrap()).unwrap(),
        date_added: unix_to_timestamp(row[1].as_integer().unwrap()),
        last_seen:  unix_to_timestamp(row[2].as_integer().unwrap()),
      })
    }
    return peers;
  }

  fn get_peers_for_hash(&self, hash: &Hash) -> Vec<Peer> {
    let mut cursor = self
      .conn
      .prepare(
        "
      SELECT address, date_added, last_seen
      FROM hashes h
        INNER JOIN peer_hashes ph ON (h.pk = ph.hash_pk)
        LEFT JOIN peers p ON (p.pk = ph.peer_pk)
      WHERE hash = ?;
    ",
      )
      .unwrap()
      .into_cursor();
    cursor.bind(&[Value::Binary(hash.0.clone())]).unwrap();
    let mut peers = Vec::new();
    while let Some(row) = cursor.next().unwrap() {
      peers.push(Peer {
        address:    PeerAddr::parse(row[0].as_string().unwrap()).unwrap(),
        date_added: unix_to_timestamp(row[1].as_integer().unwrap()),
        last_seen:  unix_to_timestamp(row[2].as_integer().unwrap()),
      })
    }
    return peers;
  }

  fn get_hashes(&self) -> Vec<(Hash, usize)> {
    let mut cursor = self
      .conn
      .prepare(
        "
      SELECT hash, COUNT(peer_pk)
      FROM hashes h
        INNER JOIN peer_hashes ph ON (h.pk = ph.hash_pk)
      GROUP BY (hash);
    ",
      )
      .unwrap()
      .into_cursor();
    let mut hashes = Vec::new();
    while let Some(row) = cursor.next().unwrap() {
      hashes.push((
        Hash(row[0].as_binary().unwrap().to_vec()),
        row[1].as_integer().unwrap() as usize,
      ))
    }
    return hashes;
  }

  fn get_peer_count(&self) -> usize {
    let mut cursor = self
      .conn
      .prepare("SELECT COUNT(pk) FROM peers;")
      .unwrap()
      .into_cursor();
    if let Some(row) = cursor.next().unwrap() {
      return row[0].as_integer().unwrap() as usize;
    } else {
      return 0;
    }
  }

  fn get_hash_count(&self) -> usize {
    let mut cursor = self
      .conn
      .prepare("SELECT COUNT(pk) FROM hashes;")
      .unwrap()
      .into_cursor();
    if let Some(row) = cursor.next().unwrap() {
      return row[0].as_integer().unwrap() as usize;
    } else {
      return 0;
    }
  }

  fn cleanup_peers(&mut self, timestamp: SystemTime) -> usize {
    let mut statement = self
      .conn
      .prepare(
        "
      DELETE FROM peer_hashes WHERE peer_pk IN (SELECT pk FROM peers WHERE last_seen < :timestamp);
      DELETE FROM peers WHERE last_seen < :timestamp;
    ",
      )
      .unwrap();
    statement
      .bind_by_name(":timestamp", timestamp_to_unix(timestamp))
      .unwrap();
    statement.next().unwrap();

    self.conn.change_count()
  }

  fn cleanup_hashes(&mut self) -> usize {
    self
      .conn
      .execute(
        "
      DELETE FROM hashes
      WHERE pk IN (
        SELECT pk FROM (
          SELECT hash_pk pk, COUNT(peer_pk) count FROM peer_hashes
        )
        WHERE count = 0
      );
    ",
      )
      .unwrap();

    self.conn.change_count()
  }
}
