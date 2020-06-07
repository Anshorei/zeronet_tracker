
use rocket::{get, routes, State};
use log::*;
use crate::peer_handler::SharedState;
use std::sync::{Arc, Mutex};
use maud::{html, Markup};

struct StateWrapper {
  shared_state: Arc<Mutex<SharedState>>,
}

pub fn run(shared_state: Arc<Mutex<SharedState>>) {
  info!("Starting server at localhost:8000");
  let state = StateWrapper {
    shared_state,
  };
  rocket::ignite()
    .mount("/", routes![overview])
    .manage(state)
    .launch();
}

#[get("/")]
fn overview(state: State<StateWrapper>) -> Markup {
  let shared_state = state.shared_state.lock().unwrap();
  html! {
    h1 { "ZeroNet Tracker" }
    p {
      "Peers: " (shared_state.peer_count())
    }
    p {
      "Hashes: " (shared_state.hash_count())
    }
  }
}
