use std::env;
use std::path::Path;

use clap::{crate_name, crate_version, App, SubCommand};

use crate::compile::compile::compile;
use crate::vm::deploy::{kill, status, new, upgrade};
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
      .about("Compiles the given source file (.ln, .amm, .aga) to a new output file (.amm, .aga, .agz, .agc, .js)")
      .arg_from_usage("<INPUT> 'Specifies the input file to load'")
      .arg_from_usage("<OUTPUT> 'Specifies the output file to generate'"))
    .subcommand(SubCommand::with_name("install")
      .about("Install '/dependencies' from '.dependencies.ln'")
      .version(crate_version!()))
    .subcommand(SubCommand::with_name("deploy-new")
      .about("Compiles a source ln file and deploys an app to the AWS account described in alan-deploy.json")
      .arg_from_usage("<FILE> 'Specifies the ln file to deploy'"))
    .subcommand(SubCommand::with_name("deploy-kill")
      .about("Kills an Alan app with the provided id from the AWS account described in alan-deploy.json")
      .arg_from_usage("<APP_ID> 'Specifies the alan app to kill'")
      .arg_from_usage("<FILE> 'Specifies the source ln file to deploy'"))
    .subcommand(SubCommand::with_name("deploy-status")
      .about("Displays all the Alan apps from the AWS account described in alan-deploy.json"))
    .subcommand(SubCommand::with_name("deploy-upgrade")
      .about("Compiles a source ln file and deploys it to an existing Alan app in the AWS account described in alan-deploy.json")
      .arg_from_usage("<APP_ID> 'Specifies the alan app to upgrade'")
      .arg_from_usage("<FILE> 'Specifies a source ln file to deploy'"))
    .get_matches();

  if let Some(matches) = matches.subcommand_matches("run") {
    let agc_file = matches.value_of("FILE").unwrap();
    let fp = &format!(
      "{:}/{:}",
      env::current_dir().ok().unwrap().to_str().unwrap(),
      agc_file
    );
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
      println!(
        "{} does not exist. Dependencies can only be installed for {}",
        source_file, source_file
      );
      std::process::exit(1);
    }
  } else if let Some(_matches) = matches.subcommand_matches("deploy-new") {
    // let ln_file = matches.value_of("FILE").unwrap();
    new();
  } else if let Some(_matches) = matches.subcommand_matches("deploy-kill") {
    let app_id = matches.value_of("APP_ID").unwrap();
    kill(app_id);
  } else if let Some(_matches) = matches.subcommand_matches("deploy-status") {
    status();
  } else if let Some(_matches) = matches.subcommand_matches("deploy-upgrade") {
    let app_id = matches.value_of("APP_ID").unwrap();
    // let ln_file = matches.value_of("FILE").unwrap();
    upgrade(app_id);
  } else if let Some(source_file) = matches.value_of("SOURCE") {
    compile_and_run(source_file);
  }
}
