use crate::shared_state::SharedState;
use log::*;
use maud::{html, Markup, PreEscaped};
use rocket::{get, routes, State};
use std::sync::{Arc, Mutex};
use std::time::Duration;

struct StateWrapper {
	shared_state: Arc<Mutex<SharedState>>,
}

pub fn run(shared_state: Arc<Mutex<SharedState>>) {
	info!("Starting server at localhost:8000");
	let state = StateWrapper { shared_state };
	rocket::ignite()
		.mount("/", routes![overview, peers, hashes])
		.manage(state)
		.launch();
}

#[get("/")]
fn overview(state: State<StateWrapper>) -> Markup {
	let shared_state = state.shared_state.lock().unwrap();
	let uptime = shared_state.start_time.elapsed().as_secs_f64() / 60f64 / 60f64;
	html! {
		h1 { "ZeroNet Tracker" }
		p { "Version: v" (crate::VERSION) }
		p { "Uptime: " (format!("{:.2}", uptime)) "h" }
		p { "Connections: " (shared_state.connections) }
		p {
			a href="/peers" { "Peers: " (shared_state.peer_count()) }
		}
		p {
			a href="/hashes" { "Hashes: " (shared_state.hash_count()) }
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
			@for peer in shared_state.peers.values() {
				li { (peer.address.to_string()) }
			}
		}
	}
}

#[get("/hashes")]
fn hashes(state: State<StateWrapper>) -> Markup {
	let shared_state = state.shared_state.lock().unwrap();
	let hashes = shared_state
		.hash_to_peer
		.iter()
		.map(|(hash, peers)| (base64::encode(hash), peers.len()));
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
