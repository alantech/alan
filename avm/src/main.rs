use std::env;
use std::fs::read;
use std::path::Path;

use anycloud::deploy;
use anycloud::oauth::authenticate;
use base64;
use clap::{crate_name, crate_version, App, AppSettings, SubCommand};
use tokio::runtime::Builder;

use crate::compile::compile::compile;
use crate::daemon::daemon::{start, CLUSTER_SECRET};
use crate::vm::run::run_file;
use crate::vm::telemetry;

mod compile;
mod daemon;
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
    if let Err(ee) = run_file(&fp, true).await {
      eprintln!("{}", ee);
      return 2;
    };
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
      .about("Deploy .agz files to one of the Deploy Configs from anycloud.json")
      .setting(AppSettings::SubcommandRequiredElseHelp)
      .subcommand(SubCommand::with_name("new")
        .about("Deploys an .agz file to a new app with one of the Deploy Configs from anycloud.json")
        .arg_from_usage("<AGZ_FILE> 'Specifies the .agz file to deploy'")
      )
      .subcommand(SubCommand::with_name("info")
        .about("Displays all the Apps deployed with the Deploy Configs from anycloud.json")
      )
      .subcommand(SubCommand::with_name("terminate")
        .about("Terminate an App hosted in one of the Deploy Configs from anycloud.json")
      )
      .subcommand(SubCommand::with_name("upgrade")
        .about("Deploys your repository to an existing App hosted in one of the Deploy Configs from anycloud.json")
        .arg_from_usage("<AGZ_FILE> 'Specifies the .agz file to deploy'")
      )
      .subcommand(SubCommand::with_name("config")
        .about("Manage Deploy Configs used by Apps from the anycloud.json in the current directory")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("add")
          .about("Add a new Deploy Config to the anycloud.json in the current directory and creates the file if it doesn't exist.")
        )
        .subcommand(SubCommand::with_name("list")
          .about("List all the Deploy Configs from the anycloud.json in the current directory")
        )
        .subcommand(SubCommand::with_name("edit")
          .about("Edit an existing Deploy Config from the anycloud.json in the current directory")
        )
        .subcommand(SubCommand::with_name("remove")
          .about("Remove an existing Deploy Config from the anycloud.json in the current directory")
        )
      )
      .subcommand(SubCommand::with_name("credentials")
        .about("Manage all Credentials used by Deploy Configs from the credentials file at ~/.anycloud/credentials.json")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(SubCommand::with_name("add")
          .about("Add a new Credentials")
        )
        .subcommand(SubCommand::with_name("list")
          .about("List all the available Credentials")
        )
        .subcommand(SubCommand::with_name("edit")
          .about("Edit an existing Credentials")
        )
        .subcommand(SubCommand::with_name("remove")
          .about("Remove an existing Credentials")
        )
      )
    )
    .subcommand(SubCommand::with_name("daemon")
      .about("Run an .agz file in daemon mode. Used on deploy within cloud provider VMs.")
      .arg_from_usage("<APP_ID> 'Specifies the alan app to upgrade'")
      .arg_from_usage("<AGZ_B64> 'Specifies the .agz program as a base64 encoded string'")
      .arg_from_usage("<DEPLOY_TOKEN> 'Specifies the deploy token'")
      .arg_from_usage("<DOMAIN> 'Specifies the application domain'")
      .arg_from_usage("<PRIVATE_KEY> -k, --private-key=<PRIV_KEY_B64> 'A base64 encoded private key for HTTPS mode'")
      .arg_from_usage("<CERT_B64> -c, --certificate=<CERT_B64> 'A base64 encoded certificate for HTTPS mode'")
      .arg_from_usage("<CLUSTER_SECRET> -s, --cluster-secret=<CLUSTER_SECRET> 'A secret string to constrain access to the control port'")
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
      ("run", Some(matches)) => {
        let agc_file = matches.value_of("FILE").unwrap();
        let fp = &format!(
          "{:}/{:}",
          env::current_dir().ok().unwrap().to_str().unwrap(),
          agc_file
        );
        telemetry::log("avm-run").await;
        if let Err(ee) = run_file(&fp, false).await {
          eprintln!("{}", ee);
          std::process::exit(2);
        };
      }
      ("compile", Some(matches)) => {
        let source_file = matches.value_of("INPUT").unwrap();
        let dest_file = matches.value_of("OUTPUT").unwrap();
        std::process::exit(compile(&source_file, &dest_file, false));
      }
      ("install", _) => {
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
      }
      ("deploy", Some(sub_matches)) => {
        authenticate().await;
        match sub_matches.subcommand() {
          ("new", Some(matches)) => {
            let agz_file = matches.value_of("AGZ_FILE").unwrap();
            deploy::new(get_agz_b64(agz_file), None, None).await;
          }
          ("terminate", _) => deploy::terminate().await,
          ("upgrade", Some(matches)) => {
            let agz_file = matches.value_of("AGZ_FILE").unwrap();
            deploy::upgrade(get_agz_b64(agz_file), None, None).await;
          }
          ("info", _) => deploy::info().await,
          ("credentials", Some(sub_matches)) => {
            match sub_matches.subcommand() {
              ("add", _) => {
                deploy::add_cred().await;
              }
              ("edit", _) => deploy::edit_cred().await,
              ("list", _) => deploy::list_creds().await,
              ("remove", _) => deploy::remove_cred().await,
              // rely on AppSettings::SubcommandRequiredElseHelp
              _ => {}
            }
          }
          ("config", Some(sub_matches)) => {
            match sub_matches.subcommand() {
              ("add", _) => deploy::add_deploy_config().await,
              ("list", _) => deploy::list_deploy_configs().await,
              ("edit", _) => deploy::edit_deploy_config().await,
              ("remove", _) => deploy::remove_deploy_config().await,
              // rely on AppSettings::SubcommandRequiredElseHelp
              _ => {}
            }
          }
          // rely on AppSettings::SubcommandRequiredElseHelp
          _ => {}
        }
      }
      ("daemon", Some(matches)) => {
        let app_id = matches.value_of("APP_ID").unwrap();
        let agz_b64 = matches.value_of("AGZ_B64").unwrap();
        let deploy_token = matches.value_of("DEPLOY_TOKEN").unwrap();
        let domain = matches.value_of("DOMAIN").unwrap();
        let priv_key_b64 = matches.value_of("PRIVATE_KEY").unwrap();
        let cert_b64 = matches.value_of("CERT_B64").unwrap();
        let cluster_secret = matches.value_of("CLUSTER_SECRET").unwrap();
        CLUSTER_SECRET
          .set(Some(cluster_secret.to_string()))
          .unwrap();
        start(
          app_id,
          agz_b64,
          deploy_token,
          domain,
          priv_key_b64,
          cert_b64,
        )
        .await;
      }
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
