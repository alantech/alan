use std::env;
use std::fs::read;
use std::path::Path;

use anycloud::deploy::{info, get_config, new, terminate, upgrade};
use base64;
use clap::{crate_name, crate_version, App, AppSettings, SubCommand};
use serde_json::json;
use tokio::runtime::Builder;

use crate::compile::compile::compile;
use crate::daemon::daemon::start;
use crate::vm::telemetry;
use crate::vm::run::run_file;

mod daemon;
mod compile;
mod vm;

fn get_agz_b64(agz_file: &str) -> String {
  let agz = read(agz_file).expect(&format!("No agz file found in {}", agz_file));
  return base64::encode(agz);
}

async fn compile_and_run(source_file: &str) -> i32 {
  let dest_file = "temp.agc";
  let status_code = compile(&source_file, &dest_file, true);
  if status_code == 0 {
    let mut path = env::current_dir().unwrap();
    path.push(dest_file);
    let fp = path.into_os_string().into_string().unwrap();
    run_file(&fp, true).await;
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
      .about("Deploy .agz files to one of the deploy configs at ~/.anycloud/deploy.json")
      .setting(AppSettings::SubcommandRequiredElseHelp)
      .subcommand(SubCommand::with_name("new")
        .about("Deploys an .agz file to a new app with one of the deploy configs at ~/.anycloud/deploy.json")
        .arg_from_usage("<AGZ_FILE> 'Specifies the .agz file to deploy'")
        .arg_from_usage("[DEPLOY_NAME] 'Specifies the name of the deploy config to use, or the first definition if not specified'")
        .arg_from_usage("-a, --app-id=[APP_ID] 'Specifies an optional application identifier'")
      )
      .subcommand(SubCommand::with_name("info")
        .about("Displays all the apps deployed with  described in the deploy config at ~/.anycloud/deploy.json")
      )
      .subcommand(SubCommand::with_name("terminate")
        .about("Terminate an app with the provided id hosted in one of the deploy configs at ~/.anycloud/deploy.json")
        .arg_from_usage("<APP_ID> 'Specifies the alan app to terminate'")
      )
      .subcommand(SubCommand::with_name("upgrade")
        .about("Deploys your repository to an existing app hosted in one of the deploy configs at ~/.anycloud/deploy.json")
        .arg_from_usage("<APP_ID> 'Specifies the alan app to upgrade'")
        .arg_from_usage("<AGZ_FILE> 'Specifies the .agz file to deploy'")
      )
    )
    .subcommand(SubCommand::with_name("daemon")
      .about("Run an .agz file in daemon mode. Used on deploy within cloud provider VMs.")
      .arg_from_usage("<APP_ID> 'Specifies the alan app to upgrade'")
      .arg_from_usage("<AGZ_B64> 'Specifies the .agz program as a base64 encoded string'")
      .arg_from_usage("<DEPLOY_TOKEN> 'Specifies the deploy token'")
      .arg_from_usage("<DOMAIN> 'Specifies the application domain'")
    )
    .arg_from_usage("[SOURCE] 'Specifies a source ln file to compile and run'");

  let matches = app.clone().get_matches();

  let rt = Builder::new_multi_thread()
    .enable_time()
    .enable_io()
    .build()
    .unwrap();

  rt.block_on(async move {
    match matches.subcommand() {
      ("run",  Some(matches)) => {
        let agc_file = matches.value_of("FILE").unwrap();
        let fp = &format!(
          "{:}/{:}",
          env::current_dir().ok().unwrap().to_str().unwrap(),
          agc_file
        );
        telemetry::log("avm-run").await;
        run_file(&fp, false).await;
      },
      ("compile",  Some(matches)) => {
        let source_file = matches.value_of("INPUT").unwrap();
        let dest_file = matches.value_of("OUTPUT").unwrap();
        std::process::exit(compile(&source_file, &dest_file, false));
      },
      ("install",  _) => {
        let source_file = ".dependencies.ln";
        if Path::new(source_file).exists() {
          std::process::exit(compile_and_run(source_file).await);
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
            let config = get_config();
            let agz_file = matches.value_of("AGZ_FILE").unwrap();
            let deploy_name = matches.value_of("DEPLOY_NAME").unwrap_or(
              config.keys().take(1).next().unwrap()
            );
            let app_id = matches.value_of("app-id");
            let body = json!({
              "deployConfig": config,
              "deployName": deploy_name,
              "agzB64": get_agz_b64(agz_file),
              "appId": app_id,
            });
            new(body).await;
          },
          ("terminate",  Some(matches)) => {
            let app_id = matches.value_of("APP_ID").unwrap();
            terminate(app_id).await;
          },
          ("upgrade",  Some(matches)) => {
            let config = get_config();
            let cluster_id = matches.value_of("APP_ID").unwrap();
            let agz_file = matches.value_of("AGZ_FILE").unwrap();
            let body = json!({
              "clusterId": cluster_id,
              "deployConfig": config,
              "agzB64": get_agz_b64(agz_file),
            });
            upgrade(body).await;
          },
          ("info",  _) => {
            info().await;
          },
          // rely on AppSettings::SubcommandRequiredElseHelp
          _ => {}
        }
      },
      ("daemon",  Some(matches)) => {
        let app_id = matches.value_of("APP_ID").unwrap();
        let agz_b64 = matches.value_of("AGZ_B64").unwrap();
        let deploy_token = matches.value_of("DEPLOY_TOKEN").unwrap();
        let domain = matches.value_of("DOMAIN").unwrap();
        start(app_id, agz_b64, deploy_token, domain).await;
      },
      _ => {
        // AppSettings::SubcommandRequiredElseHelp does not cut it here
        if let Some(source_file) = matches.value_of("SOURCE") {
          let path = Path::new(source_file);
          if path.extension().is_some() {
            std::process::exit(compile_and_run(source_file).await);
          }
        }
        app.clone().print_help().unwrap();
      }
    }
  });
}
