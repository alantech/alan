use std::env;
use std::path::Path;

use clap::{crate_name, crate_version, App, AppSettings, SubCommand};

use crate::compile::compile::compile;
use crate::daemon::daemon::{start};
use crate::vm::deploy::{kill, status, new, upgrade};
use crate::vm::run::exec;

mod daemon;
mod compile;
mod vm;

fn compile_and_run(source_file: &str) -> i32 {
  let dest_file = "temp.agc";
  let status_code = compile(&source_file, &dest_file, true);
  if status_code == 0 {
    let mut path = env::current_dir().unwrap();
    path.push(dest_file);
    let fp = path.into_os_string().into_string().unwrap();
    exec(&fp, true);
  }
  return status_code;
}

fn main() {
  let app = App::new(crate_name!())
    .version(crate_version!())
    .about("Compile, run and deploy Alan")
    .subcommand(SubCommand::with_name("run")
      .about("Runs compiled .agc files")
      .version(crate_version!())
      .arg_from_usage("<FILE> 'Specifies the file to load'")
    )
    .subcommand(SubCommand::with_name("compile")
      .about("Compiles the given source file (.ln, .amm, .aga) to a new output file (.amm, .aga, .agz, .agc, .js)")
      .arg_from_usage("<INPUT> 'Specifies the input file to load'")
      .arg_from_usage("<OUTPUT> 'Specifies the output file to generate'")
    )
    .subcommand(SubCommand::with_name("install")
      .about("Install '/dependencies' from '.dependencies.ln'")
    )
    .subcommand(SubCommand::with_name("deploy")
      .about("Deploy .agz files to the cloud provider described in the deploy config at ~/.alan/deploy.json")
      .setting(AppSettings::SubcommandRequiredElseHelp)
      .subcommand(SubCommand::with_name("new")
        .about("Deploys an .agz file to a new app in one of the cloud providers described in the deploy config at ~/.alan/deploy.json")
        .arg_from_usage("<AGZ_FILE> 'Specifies the .agz file to deploy'")
        .arg_from_usage("<CLOUD_ALIAS> 'Specifies the cloud provider to deploy to based on its alias'")
      )
      .subcommand(SubCommand::with_name("status")
        .about("Displays all the apps deployed in the cloud provider described in the deploy config at ~/.alan/deploy.json")
      )
      .subcommand(SubCommand::with_name("kill")
        .about("Kills an app with the provided id in the cloud provider described in the deploy config at ~/.alan/deploy.json")
        .arg_from_usage("<APP_ID> 'Specifies the alan app to kill'")
      )
      .subcommand(SubCommand::with_name("upgrade")
        .about("Deploys an .agz file to an existing app in the cloud provider described in the deploy config at ~/.alan/deploy.json")
        .arg_from_usage("<APP_ID> 'Specifies the alan app to upgrade'")
        .arg_from_usage("<AGZ_FILE> 'Specifies the .agz file to deploy'")
      )
    )
    .subcommand(SubCommand::with_name("daemon")
      .about("Run an .agz file in daemon mode. Used on deploy within cloud provider VMs.")
      .arg_from_usage("<APP_ID> 'Specifies the alan app to upgrade'")
      .arg_from_usage("<AGZ_FILE> 'Specifies the .agz file to deploy'")
      .arg_from_usage("<DEPLOY_TOKEN> 'Specifies the .agz file to deploy'")
    )
    .arg_from_usage("[SOURCE] 'Specifies a source ln file to compile and run'");

  let matches = app.clone().get_matches();

  match matches.subcommand() {
    ("run",  Some(matches)) => {
      let agc_file = matches.value_of("FILE").unwrap();
      let fp = &format!(
        "{:}/{:}",
        env::current_dir().ok().unwrap().to_str().unwrap(),
        agc_file
      );
      exec(&fp, false);
    },
    ("compile",  Some(matches)) => {
      let source_file = matches.value_of("INPUT").unwrap();
      let dest_file = matches.value_of("OUTPUT").unwrap();
      std::process::exit(compile(&source_file, &dest_file, false));
    },
    ("install",  _) => {
      let source_file = ".dependencies.ln";
      if Path::new(source_file).exists() {
        std::process::exit(compile_and_run(source_file));
      } else {
        println!(
          "{} does not exist. Dependencies can only be installed for {}",
          source_file, source_file
        );
        std::process::exit(1);
      }
    },
    ("deploy", Some(sub_matches)) => {
      match sub_matches.subcommand() {
        ("new",  Some(matches)) => {
          let agz_file = matches.value_of("AGZ_FILE").unwrap();
          let cloud_alias = matches.value_of("CLOUD_ALIAS").unwrap();
          new(agz_file, cloud_alias);
        },
        ("kill",  Some(matches)) => {
          let app_id = matches.value_of("APP_ID").unwrap();
          kill(app_id);
        },
        ("upgrade",  Some(matches)) => {
          let app_id = matches.value_of("APP_ID").unwrap();
          let agz_file = matches.value_of("AGZ_FILE").unwrap();
          upgrade(app_id, agz_file);
        },
        ("status",  _) => {
          status();
        },
        // rely on AppSettings::SubcommandRequiredElseHelp
        _ => {}
      }
    },
    ("daemon",  Some(matches)) => {
      let agz_file = matches.value_of("AGZ_FILE").unwrap();
      let app_id = matches.value_of("APP_ID").unwrap();
      let deploy_key = matches.value_of("DEPLOY_KEY").unwrap();
      start(agz_file, app_id, deploy_key);
    },
    _ => {
      // AppSettings::SubcommandRequiredElseHelp does not cut it here
      if let Some(source_file) = matches.value_of("SOURCE") {
        let path = Path::new(source_file);
        if path.extension().is_some() {
          std::process::exit(compile_and_run(source_file));
        }
      }
      app.clone().print_help().unwrap();
    }
  }
}
