// This build script depends on `alan-compile` being built and compiles the anycloud.ln file so it
// can be included as raw data within the anycloud binary
use std::process::Command;

fn main() {
  // Tell Cargo that if the anycloud.ln or alan-comple files change, rerun this build script
  println!("cargo:rerun-if-changed=alan/anycloud.ln");
  println!("cargo:rerun-if-changed=../../compiler/alan-compile");
  let output = Command::new("sh")
    .arg("-c")
    .arg("cd alan && ../../../compiler/alan-compile anycloud.ln anycloud.agz")
    .output()
    .unwrap();

  if output.status.code().unwrap() != 0 {
    // The `alan-compile` is not built. We'll guarantee that it does for building
    // the `anycloud` binary, but for those using this repo as a library, we'll just bypass this
    // piece since only `main.rs` uses the `anycloud.agz` file.
    Command::new("sh")
      .arg("-c")
      .arg("cd alan && touch anycloud.agz");
  }
}
