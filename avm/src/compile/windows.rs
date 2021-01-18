use std::env;
use std::fs::{create_dir, remove_dir, remove_file, File};
use std::include_bytes;
use std::io::prelude::*;
use std::process::{id, Command};
use std::str;

pub fn compile(source_file: &str, dest_file: &str, silent: bool) -> i32 {
  let compiler = include_bytes!("..\\..\\..\\compiler\\alan-compile.exe");
  let mut dir = env::temp_dir();
  dir.push(id().to_string());
  let dir0 = dir.clone();
  let dir1 = dir.clone();
  create_dir(dir0.into_os_string()).unwrap();
  dir.push("alan-compile.exe");
  let dir2 = dir.clone(); // Bleh
  let dir3 = dir.clone(); // Bleh
  let dir4 = dir.clone(); // Bleh
  let mut f = File::create(dir).unwrap();
  f.write_all(compiler).unwrap();
  drop(f);
  let mut source_path = env::current_dir().unwrap();
  source_path.push(source_file);
  let mut dest_path = env::current_dir().unwrap();
  dest_path.push(dest_file);
  let output = Command::new("cmd")
    .arg("/C")
    .arg(format!(
      "{} {} {}",
      &dir2.into_os_string().into_string().unwrap(),
      source_path.into_os_string().into_string().unwrap(),
      dest_path.into_os_string().into_string().unwrap(),
    ))
    .output()
    .unwrap();
  remove_file(dir3.as_path()).unwrap();
  remove_dir(dir1.as_path()).unwrap();
  if output.stdout.len() > 0 && !silent {
    print!("{}", str::from_utf8(&output.stdout).unwrap());
  }
  if output.stderr.len() > 0 {
    eprint!("{}", str::from_utf8(&output.stderr).unwrap());
  }
  return output.status.code().unwrap();
}
