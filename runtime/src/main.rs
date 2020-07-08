use std::env;

use clap::{App, SubCommand, crate_name, crate_version};

use crate::benchmark::run::benchmark;
use crate::vm::run::exec;

mod vm;
mod benchmark;

fn main() {
  let matches = App::new(crate_name!())
    .version(crate_version!())
    // .author(crate_authors!(", ")) // Causes a warning, digging in clap's source it's not obvious
    .about("The Alan Runtime (and second stage compiler, soon)")
    .subcommand(SubCommand::with_name("run")
      .about("Runs compiled .agc files")
      .version(crate_version!())
      // .author(crate_authors!(", "))
      .arg_from_usage("<FILE> 'Specifies the file to load'"))
    .subcommand(SubCommand::with_name("benchmark")
      .about("Runs benchmark code")
      .version(crate_version!()))
      //.author(crate_authors!(", ")))
    .get_matches();
  if let Some(matches) = matches.subcommand_matches("run") {
    let agc_file = matches.value_of("FILE").unwrap();
    let fp = &format!("{:}/{:}", env::current_dir().ok().unwrap().to_str().unwrap(), agc_file);
    exec(&fp);
  }
  if let Some(_matches) = matches.subcommand_matches("benchmark") {
    benchmark();
  }
}