use std::env;

use clap::{App, SubCommand, crate_name, crate_version, crate_authors};

use crate::vm::run::exec;

mod vm;

fn main() {
  let matches = App::new(crate_name!())
    .version(crate_version!())
    .author(crate_authors!(", "))
    .about("The Alan Runtime (and second stage compiler, soon)")
    .subcommand(SubCommand::with_name("run")
      .about("Runs compiled .agc files")
      .version(crate_version!())
      .author(crate_authors!(", "))
      .arg_from_usage("<FILE> 'Specifies the file to load'"))
    .get_matches();
  if let Some(matches) = matches.subcommand_matches("run") {
    let agc_file = matches.value_of("FILE").unwrap();
    let fp = &format!("{:}/{:}", env::current_dir().ok().unwrap().to_str().unwrap(), agc_file);
    exec(&fp);
  }
}