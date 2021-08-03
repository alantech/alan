use std::env;
use std::fs::{create_dir, remove_dir, remove_file, File};
use std::include_bytes;
use std::io::prelude::*;
use std::process::{id, Command};
use std::str;

use tempdir::TempDir;

pub fn compile(source_file: &str, dest_file: &str, silent: bool) -> i32 {
  let compiler = include_bytes!("..\\..\\..\\compiler\\alan-compile.exe");
  let tempdir = TempDir::new(id().to_string().as_str()).unwrap();
  let alan_compile_path = tempdir.path().join("alan-compile.exe");
  let mut f = File::create(&alan_compile_path).unwrap();
  f.write_all(compiler).unwrap();
  drop(f);
  let mut source_path = env::current_dir().unwrap();
  source_path = source_path.join(source_file).unwrap();
  let mut dest_path = env::current_dir().unwrap();
  dest_path = dest_path.join(dest_file).unwrap();
  let output = Command::new("cmd")
    .arg("/C")
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
