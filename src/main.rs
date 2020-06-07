use log::*;
use std::net::{TcpListener};
use std::sync::{Arc, Mutex};

mod peer_handler;
use peer_handler::{spawn_handler, SharedState};

fn main() {
	pretty_env_logger::init_timed();

	let shared_state = SharedState::new();
	let shared_state = Arc::new(Mutex::new(shared_state));

	trace!("Initiating...");
	let address = "127.0.0.1:8002".to_string();
	let listener = TcpListener::bind(&address).unwrap();
	trace!("Listening on {}!", address);
	for stream in listener.incoming() {
		if let Ok(stream) = stream {
			spawn_handler(shared_state.clone(), stream);
		} else {
			error!("Could not handle incoming stream!");
		}
	}
}
