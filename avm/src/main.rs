use std::collections::HashMap;
use std::env;
use std::fs::read;
use std::path::Path;

use base64;
use clap::{arg, command, crate_name, crate_version, Command};
use tokio::runtime::Builder;

use crate::cloud::common::get_agz_file_b64;
use crate::cloud::deploy;
use crate::cloud::oauth::authenticate;
use crate::compile::compile;
use crate::daemon::daemon::{start, CLUSTER_SECRET, NON_HTTP};
use crate::vm::run::run_file;
use crate::vm::telemetry;

mod cloud;
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
  let app = command!(crate_name!())
    .version(crate_version!())
    .about("Compile, run and deploy Alan")
    .subcommand(Command::new("run")
      .about("Runs compiled .agc files")
      .version(crate_version!())
      .arg(arg!(<FILE> "Specifies the file to load"))
    )
    .subcommand(Command::new("compile")
      .about("Compiles the given source file (.ln, .amm, .aga) to a new output file (.amm, .aga, .agz, .agc, .js)")
      .arg(arg!(<INPUT> "Specifies the input file to load"))
      .arg(arg!(<OUTPUT> "Specifies the output file to generate"))
    )
    .subcommand(Command::new("install")
      .about("Install '/dependencies' from '.dependencies.ln'")
    )
    .subcommand(Command::new("deploy")
      .about("Deploy .agz files to one of the Deploy Configs from alandeploy.json")
      .subcommand_required(true)
      .subcommand(Command::new("new")
        .about("Deploys an .agz file to a new app with one of the Deploy Configs from alandeploy.json")
        .arg(arg!(<AGZ_FILE> "Specifies the .agz file to deploy"))
        .arg(arg!([NON_INTERACTIVE] -n --non-interactive "Enables non-interactive CLI mode useful for scripting."))
        .arg(arg!([NON_HTTP] -h --non-http "Enables non-http server deployments."))
        .arg(arg!(-a --app-name [APP_NAME] "Specifies an optional app name."))
        .arg(arg!(-c --config-name [CONFIG_NAME] "Specifies a config name, required only in non-interactive mode."))
        .arg(arg!(-f --files [COMMA_SEPARATED_NAMES] "Specifies a set of files to include in the same working directory as your app"))
      )
      .subcommand(Command::new("list")
        .about("Displays all the Apps deployed with the Deploy Configs from alandeploy.json")
      )
      .subcommand(Command::new("terminate")
        .about("Terminate an App hosted in one of the Deploy Configs from alandeploy.json")
      )
      .subcommand(Command::new("upgrade")
        .about("Deploys your repository to an existing App hosted in one of the Deploy Configs from alandeploy.json")
        .arg(arg!(<AGZ_FILE> "Specifies the .agz file to deploy"))
        .arg(arg!([NON_INTERACTIVE] -n --non-interactive "Enables non-interactive CLI mode useful for scripting."))
        .arg(arg!([NON_HTTP] -h --non-http "Enables non-http server deployments."))
        .arg(arg!(-a --app-name [APP_NAME] "Specifies an optional app name."))
        .arg(arg!(-c --config-name [CONFIG_NAME] "Specifies a config name, required only in non-interactive mode."))
        .arg(arg!(-f --files [COMMA_SEPARATED_NAMES] "Specifies a set of files to include in the same working directory as your app"))
      )
      .subcommand(Command::new("config")
        .about("Manage Deploy Configs used by Apps from the alandeploy.json in the current directory")
        .subcommand_required(true)
        .subcommand(Command::new("new")
          .about("Add a new Deploy Config to the alandeploy.json in the current directory and creates the file if it doesn't exist.")
        )
        .subcommand(Command::new("list")
          .about("List all the Deploy Configs from the alandeploy.json in the current directory")
        )
        .subcommand(Command::new("edit")
          .about("Edit an existing Deploy Config from the alandeploy.json in the current directory")
        )
        .subcommand(Command::new("remove")
          .about("Remove an existing Deploy Config from the alandeploy.json in the current directory")
        )
      )
      .subcommand(Command::new("credentials")
        .about("Manage all Credentials used by Deploy Configs from the credentials file at ~/.alan/credentials.json")
        .subcommand_required(true)
        .subcommand(Command::new("new")
          .about("Add a new Credentials")
        )
        .subcommand(Command::new("list")
          .about("List all the available Credentials")
        )
        .subcommand(Command::new("edit")
          .about("Edit an existing Credentials")
        )
        .subcommand(Command::new("remove")
          .about("Remove an existing Credentials")
        )
      )
    )
    .subcommand(Command::new("daemon")
      .about("Run an .agz file in daemon mode. Used on deploy within cloud provider VMs.")
      .arg(arg!(<CLUSTER_SECRET> -s --cluster-secret <CLUSTER_SECRET> "A secret string to constrain access to the control port"))
      .arg(arg!(-f --agz-file [AGZ_FILE] "Specifies an optional agz file relative path for local usage"))
      .arg(arg!([NON_HTTP] -h --non-http "Specifies non-http agz execution."))
      .arg(arg!([ANYCLOUD_APP] -a --anycloud-app "Specifies an optional AnyCloud app flag for local usage")) // TODO: Eliminate this
    )
    .arg(arg!([SOURCE] "Specifies a source ln file to compile and run"));

  let matches = app.clone().get_matches();

  let rt = Builder::new_multi_thread()
    .enable_time()
    .enable_io()
    .build()
    .unwrap();

  rt.block_on(async move {
    match matches.subcommand() {
      Some(("run", matches)) => {
        let agc_file = matches.get_one::<String>("FILE").unwrap();
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
      Some(("compile", matches)) => {
        let source_file = matches.get_one::<String>("INPUT").unwrap();
        let dest_file = matches.get_one::<String>("OUTPUT").unwrap();
        std::process::exit(compile(&source_file, &dest_file, false));
      }
      Some(("install", _)) => {
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
      Some(("deploy", sub_matches)) => {
        match sub_matches.subcommand() {
          Some(("new", matches)) => {
            let non_interactive = matches.get_flag("NON_INTERACTIVE");
            let non_http = matches.get_flag("NON_HTTP");
            authenticate(non_interactive).await;
            let agz_file = matches.get_one::<String>("AGZ_FILE").unwrap();
            let app_name = matches.get_one::<String>("app-name").map(String::from);
            let config_name = matches.get_one::<String>("config-name").map(String::from);
            let files = matches.get_one::<String>("files");
            let mut files_b64 = HashMap::new();
            if let Some(files) = files {
              let names = files.split(",");
              for name in names {
                files_b64.insert(name.to_string(), get_agz_file_b64(name.to_string()).await);
              }
            }
            deploy::new(
              get_agz_b64(agz_file),
              files_b64,
              app_name,
              config_name,
              non_interactive,
              non_http,
            )
            .await;
          }
          Some(("terminate", matches)) => {
            let non_interactive = matches.get_flag("NON_INTERACTIVE");
            authenticate(non_interactive).await;
            let app_name = matches.get_one::<String>("app-name").unwrap();
            let config_name = matches.get_one::<String>("config-name").unwrap();
            deploy::terminate(
              Some(app_name.clone()),
              Some(config_name.clone()),
              non_interactive,
            )
            .await // TODO: Change the signature of this function to use the new types
          }
          Some(("upgrade", matches)) => {
            let non_interactive = matches.get_flag("NON_INTERACTIVE");
            let non_http = matches.get_flag("NON_HTTP");
            authenticate(non_interactive).await;
            let agz_file = matches.get_one::<String>("AGZ_FILE").unwrap();
            let app_name = matches.get_one::<String>("app-name").unwrap();
            let config_name = matches.get_one::<String>("config-name").unwrap();
            let files = matches.get_one::<String>("files");
            let mut files_b64 = HashMap::new();
            if let Some(files) = files {
              let names = files.split(",");
              for name in names {
                files_b64.insert(name.to_string(), get_agz_file_b64(name.to_string()).await);
              }
            }
            deploy::upgrade(
              get_agz_b64(agz_file),
              files_b64,
              Some(app_name.clone()), // TODO: Change the signature of this function to use the new types
              Some(config_name.clone()),
              non_interactive,
              non_http,
            )
            .await;
          }
          Some(("list", _)) => {
            authenticate(false).await;
            deploy::info().await
          }
          Some(("credentials", sub_matches)) => {
            authenticate(false).await;
            match sub_matches.subcommand() {
              Some(("new", _)) => {
                deploy::add_cred(None).await;
              }
              Some(("edit", _)) => deploy::edit_cred().await,
              Some(("list", _)) => deploy::list_creds().await,
              Some(("remove", _)) => deploy::remove_cred().await,
              // rely on AppSettings::SubcommandRequiredElseHelp
              _ => {}
            }
          }
          Some(("config", sub_matches)) => {
            authenticate(false).await;
            match sub_matches.subcommand() {
              Some(("new", _)) => deploy::add_deploy_config().await,
              Some(("list", _)) => deploy::list_deploy_configs().await,
              Some(("edit", _)) => deploy::edit_deploy_config().await,
              Some(("remove", _)) => deploy::remove_deploy_config().await,
              // rely on AppSettings::SubcommandRequiredElseHelp
              _ => {}
            }
          }
          // rely on AppSettings::SubcommandRequiredElseHelp
          _ => {
            authenticate(false).await;
          }
        }
      }
      Some(("daemon", matches)) => {
        let non_http = matches.get_flag("NON_HTTP");
        let cluster_secret = matches.get_one::<String>("CLUSTER_SECRET").unwrap();
        let local_agz_b64 = match matches.get_one::<String>("agz-file") {
          Some(agz_file_path) => Some(get_agz_file_b64(agz_file_path.to_string()).await),
          None => None,
        };
        let is_local_anycloud_app = matches.get_flag("ANYCLOUD_APP");
        NON_HTTP.set(non_http).unwrap();
        CLUSTER_SECRET
          .set(Some(cluster_secret.to_string()))
          .unwrap();
        start(is_local_anycloud_app, local_agz_b64).await;
      }
      _ => {
        // AppSettings::SubcommandRequiredElseHelp does not cut it here
        if let Some(source_file) = matches.get_one::<String>("SOURCE") {
          let path = Path::new(source_file);
          if path.extension().is_some() {
            std::process::exit(compile_and_run(&source_file).await);
          }
        }
        app.clone().print_help().unwrap();
      }
    }
  });
}
