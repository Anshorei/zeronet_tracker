use std::collections::HashMap;
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
use zeronet_peerdb::{Hash, Peer};

#[cfg(feature = "metrics")]
use crate::metrics;
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
      #[cfg(feature = "metrics")]
      let start_time = SystemTime::now();

      handler.run();

      #[cfg(feature = "metrics")]
      metrics::CLOSED_CONNECTIONS.inc();
      #[cfg(feature = "metrics")]
      metrics::CONNECTION_DURATION_SECONDS
        .inc_by(SystemTime::now().duration_since(start_time).map(|d| d.as_secs_f64()).unwrap_or(0.))
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
      let cmd = req.cmd.clone();

      info!("Received {} from {}", cmd, self.address.to_string());
      let t = SystemTime::now();
      match cmd.as_str() {
        "handshake" => self.handle_handshake(req),
        "announce" => self.handle_announce(req),
        _ => self.handle_unsupported(req.req_id),
      };
      info!(
        "Handled {} from {} in {:?}",
        cmd,
        self.address.to_string(),
        SystemTime::now().duration_since(t).unwrap()
      );
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
      match Address::parse(format!("{}.onion:{}", onion, handshake.fileserver_port)) {
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
        shared_state
          .peer_db
          .remove_peer(&address)
          .expect("Could not remove peer");
      }

      let peer = shared_state
        .peer_db
        .get_peer(&address)
        .expect("Could not get peer");
      let date_added = match peer {
        Some(peer) => peer.date_added,
        None => SystemTime::now(),
      };
      let peer = Peer {
        address,
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

        if !peer_address.starts_with("127.0.0.1") && !peer_address.starts_with("192.") {
          trace!("Updating peer {}", peer_address);
          let peer_already_known = shared_state
            .peer_db
            .update_peer(&peer, &hashes)
            .expect("Could not update peer");
          match peer_already_known {
            true => info!("Updated peer {} for {} hashes", peer_address, hashes.len()),
            false => info!("Added peer {} for {} hashes", peer_address, hashes.len()),
          }
        }
      } else {
        let mut onion_hashes = HashMap::<String, Vec<Hash>>::new();
        announce
          .onions
          .iter()
          .zip(hashes.iter())
          .for_each(|(onion, hash)| {
            if let Some(hashes) = onion_hashes.get_mut(onion) {
              hashes.push(hash.clone());
            } else {
              onion_hashes.insert(onion.to_string(), vec![hash.clone()]);
            }
          });
        onion_hashes.into_iter().for_each(|(onion, hashes)| {
          match Address::parse(format!("{}.onion:{}", onion, announce.port)) {
            Ok(onion) => {
              let peer = Peer {
                address: onion,
                last_seen: SystemTime::now(),
                date_added,
              };
              let num_of_hashes = hashes.len();
              let t = SystemTime::now();
              shared_state
                .peer_db
                .update_peer(&peer, &hashes)
                .expect("Could not update peer");
              trace!(
                "Updated onion with {} hashes in {:?}",
                num_of_hashes,
                SystemTime::now().duration_since(t).unwrap(),
              );
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
          .expect("Could not get peers for hash")
          .into_iter()
          .for_each(|peer| {
            let bytes = ByteBuf::from(peer.address.pack());
            match peer.address {
              Address::IPV4(_, _) => {
                // Need to check 'ip4' for backwards compat
                if announce.need_types.contains(&"ipv4".to_string()) || announce.need_types.contains(&"ip4".to_string()) {
                  peers.ip_v4.push(bytes);
                }
              }
              Address::IPV6(_, _) => {
                if announce.need_types.contains(&"ipv6".to_string()) {
                  peers.ip_v6.push(bytes);
                }
              }
              #[cfg(feature = "tor")]
              Address::OnionV2(_, _) | Address::OnionV3(_, _) => {
                if announce.need_types.contains(&"onion".to_string()) {
                  peers.onion_v2.push(bytes);
                }
              }
              #[cfg(feature = "i2p")]
              _ => {
                // TODO: implement i2p
                unimplemented!()
              }
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
