use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use futures::executor::block_on;
use log::*;
use serde_bytes::ByteBuf;
use zeronet_protocol::{
  error::Error,
  message::{templates, Request},
  PeerAddr as Address, ZeroConnection,
};

#[cfg(feature = "metrics")]
use crate::metrics;
use crate::peer_db::{Hash, Peer};
use crate::shared_state::SharedState;

pub fn spawn_handler(shared_state: Arc<Mutex<SharedState>>, stream: TcpStream) {
  if let Ok(address) = stream.peer_addr() {
    info!("Incoming connection from {}", address);
    let address = Address::from(address);

    std::thread::spawn(move || {
      let connection =
        ZeroConnection::new(Box::new(stream.try_clone().unwrap()), Box::new(stream)).unwrap();
      let mut handler = Handler::create(shared_state.clone(), connection, address);

      #[cfg(feature = "metrics")]
      metrics::OPENED_CONNECTIONS.inc();

      handler.run();

      #[cfg(feature = "metrics")]
      metrics::CLOSED_CONNECTIONS.inc();
    });
  } else {
    error!("Could not detect address for stream.");
  }
}

struct Handler {
  peer_id:      String,
  shared_state: Arc<Mutex<SharedState>>,
  connection:   ZeroConnection,
  address:      Address,
}

impl Handler {
  pub fn create(
    shared_state: Arc<Mutex<SharedState>>,
    connection: ZeroConnection,
    address: Address,
  ) -> Handler {
    Handler {
      peer_id: String::new(),
      shared_state,
      connection,
      address,
    }
  }

  pub fn run(&mut self) {
    loop {
      trace!("Waiting for data...");
      let req = block_on(self.connection.recv());

      #[cfg(feature = "metrics")]
      metrics::REQUEST_COUNTER.inc();

      if let Err(err) = req {
        match err {
          Error::Io(_) | Error::ConnectionClosed => {
            info!("Connection terminated: {}", self.address.to_string())
          }
          _ => error!("Encountered unexpected error: {:?}", err),
        }
        break;
      }
      let req = req.unwrap();
      info!("Received request: {:?}", req.cmd);
      match req.cmd.as_str() {
        "handshake" => self.handle_handshake(req),
        "announce" => self.handle_announce(req),
        _ => self.handle_unsupported(req.req_id),
      };
    }
  }

  fn handle_handshake(&mut self, req: Request) {
    trace!("Received handshake: {:?}", req);
    let handshake: Result<templates::Handshake, _> = req.body();
    let handshake = match handshake {
      Ok(handshake) => handshake,
      Err(err) => {
        return self.handle_invalid(req.req_id, err);
      }
    };
    if let Some(onion) = handshake.onion {
      let port = self.address.get_port();
      match Address::parse(format!("{}.onion:{}", onion, port)) {
        Ok(address) => self.address = address,
        Err(err) => {
          error!("Could not parse address: {:?}", err);
          return;
        }
      }
    }

    let mut body = templates::Handshake::new();
    body.peer_id = self.peer_id.clone();
    trace!("Response: {:?}", body);
    let response = self.connection.respond(req.req_id, body);
    let result = block_on(response);
    if let Err(err) = result {
      error!("Encountered error responding to handshake: {:?}", err);
    }
  }

  fn handle_announce(&mut self, req: Request) {
    let announce: Result<templates::Announce, _> = req.body();
    let announce = match announce {
      Ok(announce) => announce,
      Err(err) => return self.handle_invalid(req.req_id, err),
    };

    let announce = announce;
    trace!("Announce: {:?}", announce);

    let mut body = templates::AnnounceResponse::default();
    {
      let mut shared_state = self.shared_state.lock().unwrap();
      let address = self.address.with_port(announce.port as u16);
      if announce.delete {
        trace!("Deleting peer {}", &address);
        shared_state.peer_db.remove_peer(&address);
      }
      let peer = shared_state.peer_db.get_peer(&address);
      let date_added = match peer {
        Some(peer) => peer.date_added,
        None => SystemTime::now(),
      };
      let peer = Peer {
        address: address,
        last_seen: SystemTime::now(),
        date_added,
      };

      let hashes: Vec<Hash> = announce
        .hashes
        .iter()
        .map(|buf| Hash(buf.clone().into_vec()))
        .collect();
      if announce.onions.is_empty() {
        let peer_address = peer.address.to_string();
        let peer_already_known = shared_state.peer_db.update_peer(peer, hashes.clone());
        match peer_already_known {
          true => info!("Updated peer {} for {} hashes", peer_address, hashes.len()),
          false => info!("Added peer {} for {} hashes", peer_address, hashes.len()),
        }
      } else {
        announce
          .onions
          .iter()
          .zip(hashes.iter())
          .for_each(|(onion, hash)| {
            match Address::parse(format!("{}.onion:{}", onion, announce.port)) {
              Ok(onion) => {
                let peer = Peer {
                  address: onion,
                  last_seen: SystemTime::now(),
                  date_added,
                };
                shared_state.peer_db.update_peer(peer, vec![hash.clone()]);
              }
              Err(_) => {}
            };
          });
        info!("Added onions for {} hashes", announce.onions.len());
      }
      let mut hash_peers = Vec::new();
      hashes.into_iter().for_each(|hash| {
        let mut peers = templates::AnnouncePeers::default();
        shared_state
          .peer_db
          .get_peers_for_hash(&hash)
          .into_iter()
          .for_each(|peer| {
            let bytes = ByteBuf::from(peer.address.pack());
            match peer.address {
              Address::IPV4(_, _) => {
                peers.ip_v4.push(bytes);
              }
              Address::IPV6(_, _) => {
                peers.ip_v6.push(bytes);
              }
              #[cfg(feature = "tor")]
              Address::OnionV2(_, _) => {
                peers.onion_v2.push(bytes);
              }
              #[cfg(feature = "tor")]
              Address::OnionV3(_, _) => {
                peers.onion_v2.push(bytes);
              }
              #[cfg(feature = "i2p")]
              _ => {}
            }
          });
        hash_peers.push(peers);
      });
      body.peers = hash_peers;
    }
    trace!("Response: {:?}", &body);

    let response = self.connection.respond(req.req_id, body);
    let result = block_on(response);
    if let Err(err) = result {
      error!("Encountered error responding to announce: {:?}", err);
    }
  }

  fn handle_invalid(&mut self, req_id: usize, err: Error) {
    error!("Handling invalid request: {:?}", err);
    let body = templates::Error {
      error: format!("Invalid data: {:?}", err),
    };
    let response = self.connection.respond(req_id, body);
    let result = block_on(response);
    if let Err(err) = result {
      error!("Encountered error returning invalid data error: {:?}", err);
    }
  }

  fn handle_unsupported(&mut self, req_id: usize) {
    let body = "Unknown request".to_string();
    let response = self.connection.respond(req_id, body);
    let result = block_on(response);
    if let Err(err) = result {
      error!(
        "Encountered error responding to unsupported request: {:?}",
        err
      );
    }
  }
}
