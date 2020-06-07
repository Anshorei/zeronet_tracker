use futures::executor::block_on;
use zeronet_protocol::ZeroConnection;
use zeronet_protocol::message::{Request, templates};
use zeronet_protocol::error::Error;
use std::collections::{HashMap, HashSet};
use std::time::{Instant, Duration};
use serde_bytes::ByteBuf;
use log::*;
use std::net::{TcpStream};
use std::sync::{Arc, Mutex};
use zeronet_protocol::Address;

pub fn spawn_handler(shared_state: Arc<Mutex<SharedState>>, stream: TcpStream) {
  if let Ok(address) = stream.peer_addr() {
    info!("Incoming connection from {}", address);
    let address = Address::from(address);

    std::thread::spawn(move || {
      let connection =
        ZeroConnection::new(address, Box::new(stream.try_clone().unwrap()), Box::new(stream)).unwrap();
      let mut handler = Handler::create(shared_state, connection);
      handler.run();
    });
  } else {
    error!("Could not detect address for stream.");
  }
}

struct Handler {
	peer_id: String,
	shared_state: Arc<Mutex<SharedState>>,
	connection: ZeroConnection,
}

impl Handler {
	pub fn create(shared_state: Arc<Mutex<SharedState>>, connection: ZeroConnection) -> Handler {
		Handler {
			peer_id: "random shit".to_string(),
			shared_state,
			connection,
		}
	}
	pub fn run(&mut self) {
		loop {
			trace!("Waiting for data...");
			let req = block_on(self.connection.recv());
			if let Err(err) = req {
				match err {
					Error::Io(err) => info!("Connection terminated! {:?}", err),
					_ => error!("Encountered unexpected error: {:?}", err),
				}
				break;
			}
			let req = req.unwrap();
			info!("Received request: {:?}", req.cmd);
			match req.cmd.as_str() {
				"handshake" => self.handle_handshake(req.req_id),
				"announce" => self.handle_announce(req),
				_ => self.handle_unsupported(req.req_id),
			};
		}
	}

	fn handle_handshake(&mut self, req_id: usize) {
		let mut body = templates::Handshake::new();
		body.peer_id = self.peer_id.clone();
		let response = self.connection.respond(req_id, body);
		let result = block_on(response);
		if let Err(err) = result {
			error!("Encountered error responding to handshake: {:?}", err);
		}
	}

	fn handle_announce(&mut self, req: Request) {
		let announce: Result<templates::Announce, _> = req.body();
		let announce = match announce {
			Ok(announce) => announce,
			Err(err) => {
				return self.handle_invalid(req.req_id, err)
			}
    };
		trace!("Announce: {:?}", announce);

		let mut body = templates::AnnounceResponse::default();
		{
			let mut shared_state = self.shared_state.lock().unwrap();
			if announce.delete {
        // TODO: remove hashes peer is no longer seeding
			}
			// TODO: get actual IP
			let address = format!("{}:{}", "127.0.0.1", announce.port);
			let peer = shared_state.peers.get(&address);
			let date_added = match peer {
				Some(peer) => peer.date_added,
				None => Instant::now()
			};
			let peer = Peer {
				address: address,
				last_seen: Instant::now(),
				date_added,
				port: announce.port,
			};
      let hashes: Vec<Vec<u8>> = announce.hashes.iter().map(|buf| buf.clone().into_vec()).collect();
      shared_state.insert_peer(peer, hashes.clone());
      let mut peers = templates::AnnouncePeers::default();
      hashes.into_iter().for_each(|hash| {
        shared_state.get_peers(&hash).into_iter().for_each(|peer| {
          if let Ok(peer) = Address::parse(peer.address) {
            let bytes = ByteBuf::from(peer.pack());
            match peer {
              Address::IPV4(_, _) => {
                peers.ip_v4.push(bytes);
              },
              Address::IPV6(_, _) => {
                peers.ip_v6.push(bytes);
              },
              Address::OnionV2(_, _) => {
                peers.onion_v2.push(bytes);
              }
              _ => {},
            }
          }
        })
      });
      body.peers = peers;
		}
    trace!("Response: {:?}", &body);

		let response = self.connection.respond(req.req_id, body);
		let result = block_on(response);
		if let Err(err) = result {
			error!("Encountered error responding to announce: {:?}", err);
		}
	}

	fn handle_invalid(&mut self, req_id: usize, err: Error) {
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
			error!("Encountered error responding to unsupported request: {:?}", err);
		}
	}
}

pub struct SharedState {
	peers: HashMap<String, Peer>,
	hashes: HashMap<Vec<u8>, Hash>,
	hash_to_peer: HashMap<Vec<u8>, HashSet<String>>,
}

impl SharedState {
	pub fn new() -> SharedState {
		SharedState {
			peers: HashMap::new(),
			hashes: HashMap::new(),
			hash_to_peer: HashMap::new(),
		}
	}
	pub fn insert_peer(&mut self, peer: Peer, hashes: Vec<Vec<u8>>) {
		let address = peer.address.clone();
		let result = self.peers.insert(address.clone(), peer);

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
			self.hash_to_peer.get_mut(hash).unwrap().insert(address.clone());
		}

		match result {
			Some(_) => trace!("Updated peer {} for {} hashes", &address, hashes.len()),
			None => trace!("Added peer {} for {} hashes", &address, hashes.len()),
		}
	}
	pub fn get_peers(&self, hash: &Vec<u8>) -> Vec<Peer> {
    let peers = self.hash_to_peer.get(hash);
    let peers = match peers {
      Some(peers) => peers.iter().collect(),
      None => vec![],
    };

    peers.iter()
      .map(|peer_id| {
        let peer = self.peers.get(*peer_id).unwrap();
        peer.clone()
      }).collect()
	}
}

#[derive(Clone)]
pub struct Peer {
	pub address: String,
	pub port: usize,
	pub date_added: Instant,
	pub last_seen: Instant,
}

#[derive(Clone)]
pub struct Hash {
	pub hash: Vec<u8>,
	pub date_added: Instant,
}
