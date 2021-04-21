extern crate protoc_bin_vendored;
extern crate protoc_rust;

use protoc_bin_vendored::protoc_bin_path;
use protoc_rust::Customize;

fn main() {
  protoc_rust::Codegen::new()
    .protoc_path(protoc_bin_path().unwrap())
    .out_dir("src/vm/protos")
    .inputs(&["src/vm/protos/HandlerMemory.proto"])
    .includes(&["src/vm/protos"])
    .customize(Customize {
      // gen_mod_rs: Some(true),
      ..Default::default()
    })
    .run()
    .expect("protoc");
}
