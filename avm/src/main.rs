use std::env;
use std::path::Path;

use clap::{App, SubCommand, crate_name, crate_version};

use crate::compile::compile::compile;
use crate::vm::run::exec;

mod compile;
mod vm;

fn compile_and_run(source_file: &str) {
  let dest_file = "temp.agc";
  let status_code = compile(&source_file, &dest_file, true);
  if status_code == 0 {
    let mut path = env::current_dir().unwrap();
    path.push(dest_file);
    let fp = path.into_os_string().into_string().unwrap();
    exec(&fp, true);
  } else {
    std::process::exit(status_code);
  }
}

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
    .subcommand(SubCommand::with_name("install")
      .about("Install '/dependencies' from '.dependencies.ln'")
      .version(crate_version!()))
    .arg_from_usage("[SOURCE] 'Specifies a source ln file to compile and run'")
    .get_matches();

  if let Some(matches) = matches.subcommand_matches("run") {
    let agc_file = matches.value_of("FILE").unwrap();
    let fp = &format!("{:}/{:}", env::current_dir().ok().unwrap().to_str().unwrap(), agc_file);
    exec(&fp, false);
  } else if let Some(matches) = matches.subcommand_matches("compile") {
    let source_file = matches.value_of("INPUT").unwrap();
    let dest_file = matches.value_of("OUTPUT").unwrap();
    std::process::exit(compile(&source_file, &dest_file, false));
  } else if let Some(_matches) = matches.subcommand_matches("install") {
    let source_file = ".dependencies.ln";
    if Path::new(source_file).exists() {
      compile_and_run(source_file);
    } else {
      println!("{} does not exist. Dependencies can only be installed for {}", source_file, source_file);
      std::process::exit(1);
    }
  } else if let Some(source_file) = matches.value_of("SOURCE") {
    compile_and_run(source_file);
  }
}