#![feature(proc_macro_hygiene, decl_macro)]
use log::*;
use std::net::TcpListener;
use std::sync::{Arc, Barrier, Mutex};

mod args;
mod janitor;
mod peer_db;
mod peer_handler;
mod shared_state;
#[cfg(test)]
mod tests;

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

fn start_listener(shared_state: Arc<Mutex<SharedState>>, port: u16) -> Arc<Barrier> {
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
  let args = args::get_arguments();
  pretty_env_logger::init_timed();

  let shared_state = SharedState::new();
  let shared_state = Arc::new(Mutex::new(shared_state));

  #[cfg(feature = "server")]
  start_server(&shared_state, args.rocket_port);
  start_janitor(&shared_state, args.interval, args.timeout);
  start_listener(shared_state, args.port).wait();
}
