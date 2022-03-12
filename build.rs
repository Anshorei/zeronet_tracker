use std::process::Command;

use rustc_version::version;

fn main() {
  let is_clean = Command::new("git")
    .args(&["diff", "--quiet"])
    .status()
    .unwrap()
    .success();

  let mut revision = String::new();

  if is_clean {
    revision = String::from_utf8_lossy(
      &Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .unwrap()
        .stdout,
    )
    .trim()
    .to_string();
  }
  println!("cargo:rustc-env=CARGO_PKG_REVISION={}", &revision);

  println!("cargo:rustc-env=CARGO_PKG_RUSTC={}", version().unwrap());
}
