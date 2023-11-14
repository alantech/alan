use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::process::{id, Command, Stdio};

use tempfile::tempdir;

#[cfg(unix)]
const COMPILER: &'static [u8] = include_bytes!("../../compiler/alan-compile");
#[cfg(windows)]
const COMPILER: &'static [u8] = include_bytes!("../../compiler/alan-compile.exe");

pub fn compile(source_file: &str, dest_file: &str, silent: bool) -> i32 {
  let tmpdir = tempdir().unwrap();
  let alan_compile_path = if cfg!(unix) {
    tmpdir.path().join("alan-compile")
  } else {
    tmpdir.path().join("alan-compile.exe")
  };
  let mut f = File::create(&alan_compile_path).unwrap();
  f.write_all(COMPILER).unwrap();
  // on unix systems we also have to set the permissions for the file
  // to mark it as executable
  #[cfg(unix)]
  {
    use std::os::unix::prelude::PermissionsExt;

    let metadata = f.metadata().unwrap();
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o744);
    f.set_permissions(permissions).unwrap();
  }
  drop(f);
  let mut source_path = env::current_dir().unwrap();
  source_path.push(source_file);
  let mut dest_path = env::current_dir().unwrap();
  dest_path.push(dest_file);
  let mut cmd = if cfg!(unix) {
    let mut cmd = Command::new("sh");
    cmd.arg("-c");
    cmd
  } else {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C");
    cmd
  };
  if !silent {
    cmd.stdout(Stdio::inherit());
  }
  cmd.stderr(Stdio::inherit());
  let output = cmd
    .arg(format!(
      "{} {} {}",
      alan_compile_path.display(),
      source_path.display(),
      dest_path.display(),
    ))
    .output()
    .unwrap();
  return output.status.code().unwrap();
}
