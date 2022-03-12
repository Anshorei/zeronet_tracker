use std::sync::{Arc, Mutex};

use clap::crate_version;
use log::*;
use maud::{html, Markup, PreEscaped};
#[cfg(feature = "metrics")]
use prometheus::{Encoder, TextEncoder};
#[cfg(feature = "metrics")]
use rocket::response::content;
use rocket::{get, routes, Config, State};
use rocket_contrib::json::Json;
use serde::Serialize;

#[cfg(feature = "metrics")]
use crate::metrics;
use crate::peer_db::get_peer_db_type;
use crate::shared_state::SharedState;

struct StateWrapper {
  shared_state: Arc<Mutex<SharedState>>,
}

pub fn run(shared_state: Arc<Mutex<SharedState>>, port: u16) {
  info!("Starting server at localhost:{}", port);
  let state = StateWrapper { shared_state };
  let mut config = Config::active().unwrap();
  config.set_port(port);

  #[cfg(not(feature = "metrics"))]
  let stats_routes = routes![];
  #[cfg(feature = "metrics")]
  let stats_routes = routes![stats_json, stats_prometheus];

  rocket::custom(config)
    .mount("/", routes![overview, peers, hashes, hash_stats])
    .mount("/stats/", stats_routes)
    .manage(state)
    .launch();
}

#[get("/")]
fn overview(state: State<StateWrapper>) -> Markup {
  let shared_state = state.shared_state.lock().unwrap();
  let uptime = shared_state.start_time.elapsed().unwrap().as_secs_f64() / 60f64 / 60f64;

  html! {
    h1 { "ZeroNet Tracker" }
    p { "Version: v" (crate_version!()) }
    p { "PeerDB: " (get_peer_db_type()) }
    p { "Uptime: " (format!("{:.2}", uptime)) "h" }
    p {
      a href="/peers" { "Peers: " (shared_state.peer_db.get_peer_count().unwrap_or(0)) }
    }
    p {
      a href="/hashes" { "Hashes: " (shared_state.peer_db.get_hash_count().unwrap_or(0)) }
    }
    (stat_links())
  }
}

#[cfg(not(feature = "metrics"))]
fn stat_links() -> Markup {
  html! {}
}

#[cfg(feature = "metrics")]
fn stat_links() -> Markup {
  html! {
    p {
      a href="/stats/json" { "Data in JSON format" }
    }
    p {
      a href="/stats/prometheus" { "Prometheus scrape URL" }
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
  let peers = shared_state
    .peer_db
    .get_peers()
    .expect("Could not get peers");

  html! {
    (PreEscaped(STYLE))
    a href="/" { ("Back") }
    h1 { "ZeroNet Tracker - Peer List" }
    ol {
      @for peer in peers.iter() {
        li { (format!("{}", peer.address)) }
      }
    }
  }
}

#[get("/hashes")]
fn hashes(state: State<StateWrapper>) -> Markup {
  let shared_state = state.shared_state.lock().unwrap();
  let hashes = shared_state
    .peer_db
    .get_hashes()
    .expect("Could not get hashes");
  let hashes = hashes
    .iter()
    .map(|(hash, peers)| (base64::encode(&hash.0), peers));
  html! {
    (PreEscaped(STYLE))
    a href="/" { ("Back") }
    h1 { "ZeroNet Tracker - Hash List" }
    ol {
      @for (hash, peers) in hashes {
        li { (format!("{} ({} peers)", hash, peers)) }
      }
    }
  }
}

#[derive(Serialize)]
struct Stats {
  opened_connections: usize,
  closed_connections: usize,
  peer_count:         usize,
  hash_count:         usize,
  uptime:             u64,
  version:            String,
}

#[cfg(feature = "metrics")]
#[get("/json")]
fn stats_json(state: State<StateWrapper>) -> Json<Stats> {
  let shared_state = state.shared_state.lock().unwrap();

  Json(Stats {
    opened_connections: metrics::OPENED_CONNECTIONS.get() as usize,
    closed_connections: metrics::CLOSED_CONNECTIONS.get() as usize,
    peer_count:         shared_state.peer_db.get_peer_count().unwrap_or(0),
    hash_count:         shared_state.peer_db.get_hash_count().unwrap_or(0),
    uptime:             shared_state.start_time.elapsed().unwrap().as_secs(),
    version:            format!("v{}", crate_version!()),
  })
}

#[cfg(feature = "metrics")]
#[get("/prometheus")]
fn stats_prometheus(state: State<StateWrapper>) -> content::Plain<Vec<u8>> {
  let shared_state = state.shared_state.lock().unwrap();

  metrics::PEER_GAUGE.set(shared_state.peer_db.get_peer_count().unwrap_or(0) as i64);
  metrics::HASH_GAUGE.set(shared_state.peer_db.get_hash_count().unwrap_or(0) as i64);

  let encoder = TextEncoder::new();
  let mut buffer = vec![];
  let metric_families = prometheus::gather();
  encoder.encode(&metric_families, &mut buffer).unwrap();

  content::Plain::<Vec<u8>>(buffer)
}

#[derive(Serialize)]
struct HashStat {
  hash:  String,
  count: usize,
}

#[get("/hash_stats")]
fn hash_stats(state: State<StateWrapper>) -> Json<Vec<HashStat>> {
  let shared_state = state.shared_state.lock().unwrap();
  let hashes = shared_state
    .peer_db
    .get_hashes()
    .expect("Could not get hashes");
  let hashes: Vec<HashStat> = hashes
    .iter()
    .map(|(hash, peers)| HashStat {
      hash:  base64::encode(&hash.0),
      count: *peers,
    })
    .collect();
  Json(hashes)
}
