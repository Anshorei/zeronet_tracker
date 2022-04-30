use std::process::Command;

use rustc_version::version;

fn main() {
  let is_clean = Command::new("git")
    .args(&["diff", "--quiet"])
    .status()
    .unwrap()
    .success();

  let commit_hash = String::from_utf8_lossy(
    &Command::new("git")
      .args(&["rev-parse", "--short", "HEAD"])
      .output()
      .unwrap()
      .stdout,
  )
  .trim()
  .to_string();

  let revision = match is_clean {
    true => commit_hash,
    false => format!("~{}", commit_hash),
  };

  println!("cargo:rustc-env=CARGO_PKG_REVISION={}", &revision);

  println!("cargo:rustc-env=CARGO_PKG_RUSTC={}", version().unwrap());
}
