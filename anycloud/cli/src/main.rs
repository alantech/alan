use std::env;

use clap::{crate_name, crate_version, App, AppSettings, SubCommand};

use anycloud::common::{
  get_app_tar_gz_b64, get_base_agz_b64, get_dockerfile_b64, get_env_file_b64,
};
use anycloud::deploy;
use anycloud::oauth::authenticate;

#[tokio::main]
pub async fn main() {
  let anycloud_agz = get_base_agz_b64();
  let desc: &str = &format!("{}", env!("CARGO_PKG_DESCRIPTION"));
  let app = App::new(crate_name!())
    .version(crate_version!())
    .about(desc)
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .subcommand(SubCommand::with_name("new")
      .about("Deploys your repository to a new App with a Deploy Config from anycloud.json")
      .arg_from_usage("-e, --env-file=[ENV_FILE] 'Specifies an optional environment file'")
    )
    .subcommand(SubCommand::with_name("list")
      .about("Displays all the Apps deployed with the Deploy Configs from anycloud.json")
    )
    .subcommand(SubCommand::with_name("terminate")
      .about("Terminate an App hosted in one of the Deploy Configs from anycloud.json")
    )
    .subcommand(SubCommand::with_name("upgrade")
      .about("Deploys your repository to an existing App hosted in one of the Deploy Configs from anycloud.json")
      .arg_from_usage("-e, --env-file=[ENV_FILE] 'Specifies an optional environment file relative path'")
    )
    .subcommand(SubCommand::with_name("config")
      .about("Manage Deploy Configs used by Apps from the anycloud.json in the current directory")
      .setting(AppSettings::SubcommandRequiredElseHelp)
      .subcommand(SubCommand::with_name("new")
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
      .subcommand(SubCommand::with_name("new")
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
    );

  authenticate().await;
  let matches = app.get_matches();
  match matches.subcommand() {
    ("new", Some(matches)) => {
      let dockerfile_b64 = get_dockerfile_b64().await;
      let app_tar_gz_b64 = get_app_tar_gz_b64().await;
      let env_b64 = match matches.value_of("env-file") {
        Some(env_file) => Some(get_env_file_b64(env_file.to_string()).await),
        None => None,
      };
      deploy::new(
        anycloud_agz,
        Some((dockerfile_b64, app_tar_gz_b64)),
        env_b64,
      )
      .await;
    }
    ("terminate", _) => deploy::terminate().await,
    ("upgrade", Some(matches)) => {
      let dockerfile_b64 = get_dockerfile_b64().await;
      let app_tar_gz_b64 = get_app_tar_gz_b64().await;
      let env_b64 = match matches.value_of("env-file") {
        Some(env_file) => Some(get_env_file_b64(env_file.to_string()).await),
        None => None,
      };
      deploy::upgrade(
        anycloud_agz,
        Some((dockerfile_b64, app_tar_gz_b64)),
        env_b64,
      )
      .await;
    }
    ("list", _) => deploy::info().await,
    ("credentials", Some(sub_matches)) => {
      match sub_matches.subcommand() {
        ("new", _) => {
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
        ("new", _) => deploy::add_deploy_config().await,
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
