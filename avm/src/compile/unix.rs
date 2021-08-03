use std::env;
use std::fs::File;
use std::include_bytes;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::process::{id, Command};
use std::str;

use tempdir::TempDir;

pub fn compile(source_file: &str, dest_file: &str, silent: bool) -> i32 {
  let compiler = include_bytes!("../../../compiler/alan-compile");
  let tempdir = TempDir::new(id().to_string().as_str()).unwrap();
  let alan_compile_path = tempdir.path().join("alan-compile");
  let mut f = File::create(&alan_compile_path).unwrap();
  f.write_all(compiler).unwrap();
  let metadata = f.metadata().unwrap();
  let mut permissions = metadata.permissions();
  permissions.set_mode(0o744);
  f.set_permissions(permissions).unwrap();
  drop(f);
  let mut source_path = env::current_dir().unwrap();
  source_path.push(source_file);
  let mut dest_path = env::current_dir().unwrap();
  dest_path.push(dest_file);
  let output = Command::new("sh")
    .arg("-c")
    .arg(format!(
      "{} {} {}",
      alan_compile_path.display(),
      source_path.display(),
      dest_path.display(),
    ))
    .output()
    .unwrap();
  if output.stdout.len() > 0 && !silent {
    print!("{}", str::from_utf8(&output.stdout).unwrap());
  }
  if output.stderr.len() > 0 {
    eprint!("{}", str::from_utf8(&output.stderr).unwrap());
  }
  return output.status.code().unwrap();
}
