#![feature(proc_macro_hygiene, decl_macro)]
use log::*;
use std::net::TcpListener;
use std::sync::{Arc, Barrier, Mutex};

mod peer_handler;
#[cfg(feature = "server")]
mod server;
mod shared_state;
mod tests;

use peer_handler::spawn_handler;
use shared_state::SharedState;

const VERSION: &str = "0.1.3";

#[cfg(feature = "server")]
fn start_server(shared_state: &Arc<Mutex<SharedState>>) {
	let moved_state = shared_state.clone();
	std::thread::spawn(move || {
		server::run(moved_state);
	});
}

#[cfg(not(feature = "server"))]
fn start_server(shared_state: &Arc<Mutex<SharedState>>) {
	info!("Compiled with server feature disabled, skipping.")
}

fn start_listener(shared_state: Arc<Mutex<SharedState>>) -> Arc<Barrier> {
	let port = std::env::var("PORT").unwrap_or("8002".to_string());

	let address = format!("127.0.0.1:{}", port);
	let listener = TcpListener::bind(&address).unwrap();
	trace!("Listening on {}!", address);
	let barrier = Arc::new(Barrier::new(2));

	let moved_barrier = barrier.clone();
	std::thread::spawn(move || {
		for stream in listener.incoming() {
			if let Ok(stream) = stream {
				spawn_handler(shared_state.clone(), stream);
			} else {
				error!("Could not handle incoming stream!");
			}
		}
		moved_barrier.wait();
	});

	barrier
}

fn main() {
	pretty_env_logger::init_timed();

	let shared_state = SharedState::new();
	let shared_state = Arc::new(Mutex::new(shared_state));

	start_server(&shared_state);
	start_listener(shared_state).wait();
}
