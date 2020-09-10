use crate::shared_state::SharedState;
use log::*;
use maud::{html, Markup, PreEscaped};
use rocket::{get, routes, State};
use rocket_contrib::json::Json;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::Serialize;
use clap::crate_version;

struct StateWrapper {
	shared_state: Arc<Mutex<SharedState>>,
}

pub fn run(shared_state: Arc<Mutex<SharedState>>) {
	info!("Starting server at localhost:8000");
	let state = StateWrapper { shared_state };
	rocket::ignite()
		.mount("/", routes![overview, peers, hashes, stats])
		.manage(state)
		.launch();
}

#[get("/")]
fn overview(state: State<StateWrapper>) -> Markup {
	let shared_state = state.shared_state.lock().unwrap();
	let uptime = shared_state.start_time.elapsed().as_secs_f64() / 60f64 / 60f64;
	html! {
		h1 { "ZeroNet Tracker" }
		p { "Version: v" (crate_version!()) }
		p { "Uptime: " (format!("{:.2}", uptime)) "h" }
		p { "Connections: " (shared_state.connections) }
		p {
			a href="/peers" { "Peers: " (shared_state.peer_db.get_peer_count()) }
		}
		p {
			a href="/hashes" { "Hashes: " (shared_state.peer_db.get_hash_count()) }
		}
	}
}

const STYLE: &str = r#"<style>
li {
  font-family: monospace;
}
</style>"#;

#[get("/peers")]
fn peers(state: State<StateWrapper>) -> Markup {
	let shared_state = state.shared_state.lock().unwrap();
	html! {
		(PreEscaped(STYLE))
		a href="/" { ("Back") }
		h1 { "ZeroNet Tracker - Peer List" }
		ol {
			@for peer in shared_state.peer_db.get_peers().iter() {
				li { (peer.address.to_string()) }
			}
		}
	}
}

#[get("/hashes")]
fn hashes(state: State<StateWrapper>) -> Markup {
	let shared_state = state.shared_state.lock().unwrap();
	let hashes = shared_state.peer_db
		.get_hashes();
	let hashes = hashes
		.iter()
		.map(|(hash, peers)| (base64::encode(hash), peers));
	html! {
		(PreEscaped(STYLE))
		a href="/" { ("Back") }
		h1 { "ZeroNet Tracker - Hash List" }
		ol {
			@for (hash, peers) in hashes {
				li { (format!("{} ({})", hash, peers)) }
			}
		}
	}
}

#[derive(Serialize)]
struct Stats {
	requests: usize,
	peer_count: usize,
	hash_count: usize,
	uptime: u64,
	version: String,
}

#[get("/stats")]
fn stats(state: State<StateWrapper>) -> Json<Stats> {
	let shared_state = state.shared_state.lock().unwrap();
	let uptime = Instant::now() - shared_state.start_time;
	Json(Stats {
		requests: shared_state.requests,
		peer_count: shared_state.peer_db.get_peer_count(),
		hash_count: shared_state.peer_db.get_hash_count(),
		uptime: uptime.as_secs(),
		version: format!("v{}", crate_version!()),
	})
}
