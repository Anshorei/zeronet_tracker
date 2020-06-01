use futures::executor::block_on;
use log::*;
use std::net::{TcpListener, TcpStream};
use zeronet_protocol::ZeroConnection;

fn handshake_response() -> serde_json::Value {
	let text = r#"{
		"crypt_supported": [],
		"fileserver_port": 0,
		"port_opened": false,
		"use_bin_type": true,
		"protocol": "v2",
		"rev": 4486,
		"target_ip": "0.0.0.0",
		"version": "0.7.1"
	}"#;
	return serde_json::from_str(text).unwrap();
}

fn announce_response() -> serde_json::Value {
	let text = r#"{
		"peers": []
	}"#;
	return serde_json::from_str(text).unwrap();
}

fn main() {
	pretty_env_logger::init_timed();

	trace!("Initiating...");
	let listener = TcpListener::bind("127.0.0.1:8002").unwrap();
	trace!("Listening!");
	for stream in listener.incoming() {
		trace!("Incoming connection");
		match stream {
			Ok(stream) => handle_peer(stream),
			_ => {}
		}
	}
}

fn handle_peer(stream: TcpStream) {
	trace!("{:?}", stream.peer_addr());

	std::thread::spawn(move || {
		let mut conn =
			ZeroConnection::new(Box::new(stream.try_clone().unwrap()), Box::new(stream)).unwrap();
		loop {
			trace!("Waiting for data...");
			let req = block_on(conn.recv());
			if req.is_err() {
				error!("Encountered error: {:?}", req.unwrap_err());
				trace!("Closing connection...");
				break;
			}
			let req = req.unwrap();
			info!("Received request: {:?}", req.cmd);
			let res = match req.cmd.as_str() {
				"handshake" => conn.respond(req.req_id, handshake_response()),
				"announce" => conn.respond(req.req_id, announce_response()),
				_ => conn.respond(req.req_id, announce_response()),
			};
			if let Err(err) = block_on(res) {
				error!("Encountered error: {:?}", err);
			} else {
				info!("Responded!");
			}
		}
	});
}
