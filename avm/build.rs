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
      ..Default::default()
    })
    .run()
    .expect("protoc");
}
