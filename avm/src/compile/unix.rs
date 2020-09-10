use std::env;
use std::fs::File;
use std::include_bytes;
use std::io::prelude::*;
use std::os::unix::fs::PermissionsExt;
use std::process::Command;
use std::str;

pub fn compile(source_file: &str, dest_file: &str) {
  let compiler = include_bytes!("../../../compiler/alan-compile");
  let mut dir = env::temp_dir();
  dir.push("alan-compile");
  let dir2 = dir.clone(); // Bleh
  let dir3 = dir.clone(); // Bleh
  let mut f = File::create(dir).unwrap();
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
  let output = Command::new("sh").arg("-c").arg(format!(
    "{} {} {}",
    &dir2.into_os_string().into_string().unwrap(),
    source_path.into_os_string().into_string().unwrap(),
    dest_path.into_os_string().into_string().unwrap(),
  )).output().unwrap();
  std::fs::remove_file(dir3.as_path()).unwrap();
  if output.stdout.len() > 0 {
    print!("{}", str::from_utf8(&output.stdout).unwrap());
  }
  if output.stderr.len() > 0 {
    eprint!("{}", str::from_utf8(&output.stderr).unwrap());
  }
  std::process::exit(output.status.code().unwrap());
}