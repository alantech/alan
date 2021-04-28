use std::process::Command;

use protoc_bin_vendored::protoc_bin_path;
use protoc_rust::Customize;

extern crate protoc_bin_vendored;
extern crate protoc_rust;

fn main() {
  // Tell Cargo that if the build.rs or HandlerMemory.proto files change, rerun this build script
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=src/vm/protos/HandlerMemory.proto");

  // Protobuf schema generation
  protoc_rust::Codegen::new()
    .protoc_path(protoc_bin_path().unwrap())
    .out_dir("src/vm/protos")
    .inputs(&["src/vm/protos/HandlerMemory.proto"])
    .includes(&["src/vm/protos"])
    .customize(Customize {
      ..Default::default()
    })
    .run()
    .expect("protoc");

  // Self signed certs for local dev
  // openssl req -newkey rsa:2048 -nodes -keyout key.pem -x509 -days 365 -out certificate.pem -subj "/C=US/ST=California/O=Alan Technologies, Inc/CN=*.anycloudapp.com"
  let subj = if cfg!(target_os = "windows") {
    "//C=US\\ST=California\\O=Alan Technologies, Inc\\CN=*.anycloudapp.com"
  } else {
    "/C=US/ST=California/O=Alan Technologies, Inc/CN=*.anycloudapp.com"
  };
  Command::new("openssl")
    .arg("req")
    .arg("-newkey")
    .arg("rsa:2048")
    .arg("-nodes")
    .arg("-keyout")
    .arg("key.pem")
    .arg("-x509")
    .arg("-days")
    .arg("365")
    .arg("-out")
    .arg("certificate.pem")
    .arg("-subj")
    .arg(subj)
    .spawn()
    .expect("Error generating self signed certificate");
}
