extern crate protoc_bin_vendored;
extern crate protoc_rust;

use protoc_bin_vendored::protoc_bin_path;
use protoc_rust::Customize;

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

  let subj = "/C=US/ST=California/O=Alan Technologies, Inc/CN=*.anycloudapp.com";
  std::process::Command::new("openssl")
    .stdout(std::process::Stdio::null())
    .stderr(std::process::Stdio::null())
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
