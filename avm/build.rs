extern crate protoc_rust;

use protoc_rust::Customize;

fn main() {
  protoc_rust::run(protoc_rust::Args {
    out_dir: "src/vm/protos",
    input: &["src/vm/protos/HandlerMemory.proto"],
    includes: &["src/vm/protos"],
    customize: Customize {
      ..Default::default()
    },
  })
  .expect("protoc");
}
