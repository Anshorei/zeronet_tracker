use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version, Arg};

pub struct Args {
  #[cfg(feature = "server")]
  pub rocket_port: u16,
  pub port:        u16,
  pub address:     String,
  pub interval:    u16,
  pub timeout:     u16,
}

fn is_u16(v: String) -> Result<(), String> {
  let res: Result<u16, _> = v.parse();
  if res.is_ok() {
    return Ok(());
  } else {
    return Err(format!("'{}' cannot be parsed to u16.", v));
  }
}

pub fn get_arguments() -> Args {
  let mut app = app_from_crate!();
  app = app
    .arg(
      Arg::with_name("address")
        .short("a")
        .long("address")
        .help("Address to listen on for peer connections.")
        .env("ADDRESS")
        .default_value("localhost"),
    )
    .arg(
      Arg::with_name("listener_port")
        .short("p")
        .long("port")
        .visible_alias("listener_port")
        .help("Port to listen on for peer connections.")
        .env("LISTENER_PORT")
        .validator(is_u16)
        .default_value("15442"),
    )
    .arg(
      Arg::with_name("janitor_interval")
        .short("i")
        .long("interval")
        .help("Interval for the janitor's cleanup of dead peers and stale hashes")
        .env("JANITOR_INTERVAL")
        .validator(is_u16)
        .default_value("60"),
    )
    .arg(
      Arg::with_name("timeout")
        .short("t")
        .long("timeout")
        .help("Number of minutes without announce before a peer is considered dead.")
        .env("PEER_TIMEOUT")
        .validator(is_u16)
        .default_value("50"),
    );

  #[cfg(feature = "server")]
  {
    app = app.arg(
      Arg::with_name("rocket_port")
        .long("rocket_port")
        .visible_alias("server_port")
        .help("Port to serve the stats on.")
        .env("ROCKET_PORT")
        .validator(is_u16)
        .default_value("15441"),
    );
  }

  let matches = app.get_matches();
  let args = Args {
    #[cfg(feature = "server")]
    rocket_port: matches.value_of("rocket_port").unwrap().parse().unwrap(),
    port:        matches.value_of("listener_port").unwrap().parse().unwrap(),
    address:     matches.value_of("address").unwrap().to_string(),
    interval:    matches
      .value_of("janitor_interval")
      .unwrap()
      .parse()
      .unwrap(),
    timeout:     matches.value_of("timeout").unwrap().parse().unwrap(),
  };

  args
}
