extern crate protoc_bin_vendored;
extern crate protoc_rust;

use protoc_bin_vendored::protoc_bin_path;
use protoc_rust::Customize;

fn main() {
  // if the alan-compile bin has changed, trigger recompilation
  if std::env::var("TARGET").unwrap().contains("windows") {
    println!("cargo:rerun-if-changed=../compiler/alan-compile.exe");
  } else {
    println!("cargo:rerun-if-changed=../compiler/alan-compile");
  }

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
}
