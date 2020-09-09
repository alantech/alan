use std::env;

use clap::{App, SubCommand, crate_name, crate_version};

use crate::benchmark::run::benchmark;
use crate::compile::compile::compile;
use crate::vm::run::exec;

mod benchmark;
mod compile;
mod vm;

fn main() {
  let matches = App::new(crate_name!())
    .version(crate_version!())
    // .author(crate_authors!(", ")) // Causes a warning, digging in clap's source it's not obvious
    .about("The Alan Compiler and VM")
    .subcommand(SubCommand::with_name("run")
      .about("Runs compiled .agc files")
      .version(crate_version!())
      // .author(crate_authors!(", "))
      .arg_from_usage("<FILE> 'Specifies the file to load'"))
    .subcommand(SubCommand::with_name("compile")
      .about("Compiles the given source file (.ln, .amm, .aga) to a new output file (.amm, .aga, .agc, .js)")
      .arg_from_usage("<INPUT> 'Specifies the input file to load'")
      .arg_from_usage("<OUTPUT> 'Specifies the output file to generate'"))
    .subcommand(SubCommand::with_name("benchmark")
      .about("Runs benchmark code")
      .version(crate_version!()))
      //.author(crate_authors!(", ")))
    .get_matches();

  if let Some(matches) = matches.subcommand_matches("run") {
    let agc_file = matches.value_of("FILE").unwrap();
    let fp = &format!("{:}/{:}", env::current_dir().ok().unwrap().to_str().unwrap(), agc_file);
    exec(&fp);
  } else if let Some(matches) = matches.subcommand_matches("compile") {
    let source_file = matches.value_of("INPUT").unwrap();
    let dest_file = matches.value_of("OUTPUT").unwrap();
    compile(&source_file, &dest_file);
  } else if let Some(_matches) = matches.subcommand_matches("benchmark") {
    benchmark();
  }
}