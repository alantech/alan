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
  let mut alan_compile_path = tempdir.path().to_owned();
  alan_compile_path.push("alan-compile");
  let mut f = File::create(&alan_compile_path).unwrap();
  f.write_all(compiler).unwrap();
  let metadata = f.metadata().unwrap();
  let mut permissions = metadata.permissions();
  permissions.set_mode(0o744);
  f.set_permissions(permissions).unwrap();
  drop(f);
  let mut source_path = env::current_dir().unwrap();
  source_path = source_path.join(source_file);
  let mut dest_path = env::current_dir().unwrap();
  dest_path = dest_path.join(dest_file);
  let output = Command::new("sh")
    .arg("-c")
    .arg(format!(
      "{} {} {}",
      alan_compile_path.as_path().display(),
      source_path.as_path().display(),
      dest_path.as_path().display(),
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
