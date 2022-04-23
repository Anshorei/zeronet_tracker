#![cfg_attr(feature = "server", feature(proc_macro_hygiene, decl_macro))]
use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use clap::{crate_name, crate_version};
use log::*;

mod args;
mod janitor;
mod peer_db;
mod peer_handler;
mod shared_state;

#[cfg(feature = "metrics")]
mod metrics;
#[cfg(feature = "server")]
mod server;

use peer_handler::spawn_handler;
use shared_state::SharedState;

#[cfg(feature = "server")]
fn start_server(shared_state: &Arc<Mutex<SharedState>>, port: u16) {
  let moved_state = shared_state.clone();
  std::thread::spawn(move || {
    server::run(moved_state, port);
  });
}

fn start_janitor(shared_state: &Arc<Mutex<SharedState>>, interval: u16, timeout: u16) {
  info!(
    "Starting janitor with: interval={}s, timeout={}m",
    interval, timeout
  );
  let moved_state = shared_state.clone();
  std::thread::spawn(move || {
    janitor::run(moved_state, interval, timeout);
  });
}

fn start_listener(shared_state: &Arc<Mutex<SharedState>>, address: String, port: u16) {
  let address_with_port = format!("{}:{}", address, port);
  info!("Starting listener on {}", address_with_port);
  let listener = TcpListener::bind(&address_with_port).unwrap();

  for stream in listener.incoming() {
    if let Ok(stream) = stream {
      spawn_handler(shared_state.clone(), stream);
    } else {
      error!("Could not handle incoming stream!");
    }
  }
}

fn main() {
  let args = args::get_arguments();
  pretty_env_logger::init_timed();
  info!("Launched {} v{}", crate_name!(), crate_version!());
  info!("PeerDB type: {}", crate::peer_db::get_peer_db_type());

  let shared_state = SharedState::new(&args);
  let shared_state = Arc::new(Mutex::new(shared_state));

  #[cfg(feature = "server")]
  start_server(&shared_state, args.rocket_port);
  start_janitor(&shared_state, args.interval, args.timeout);
  start_listener(&shared_state, args.address, args.port);
}
