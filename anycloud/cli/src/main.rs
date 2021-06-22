use std::env;
use std::future::Future;

use clap::{crate_name, crate_version, App, AppSettings, SubCommand};

use anycloud::common::{get_app_tar_gz_b64, get_dockerfile_b64, get_env_file_b64};
use anycloud::deploy;
use anycloud::oauth::authenticate;

#[tokio::main]
pub async fn main() {
  let anycloud_agz = base64::encode(include_bytes!("../alan/anycloud.agz"));
  let desc: &str = &format!("{}", env!("CARGO_PKG_DESCRIPTION"));
  let app = App::new(crate_name!())
    .version(crate_version!())
    .about(desc)
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .subcommand(SubCommand::with_name("new")
      .about("Deploys your repository to a new App with a Deploy Config from anycloud.json")
      .arg_from_usage("-e, --env-file=[ENV_FILE] 'Specifies an optional environment file'")
      .arg_from_usage("[NON_INTERACTIVE] -n, --non-interactive 'Enables non-interactive CLI mode useful for scripting.'")
      .arg_from_usage("-a, --app-name=[APP_NAME] 'Specifies an optional app name.'")
      .arg_from_usage("-c, --config-name=[CONFIG_NAME] 'Specifies a config name, required only in non-interactive mode.'")
    )
    .subcommand(SubCommand::with_name("list")
      .about("Displays all the Apps deployed with the Deploy Configs from anycloud.json")
    )
    .subcommand(SubCommand::with_name("terminate")
      .about("Terminate an App hosted in one of the Deploy Configs from anycloud.json")
      .arg_from_usage("[NON_INTERACTIVE] -n, --non-interactive 'Enables non-interactive CLI mode useful for scripting.'")
      .arg_from_usage("-a, --app-name=[APP_NAME] 'Specifies an optional app name.'")
      .arg_from_usage("-c, --config-name=[CONFIG_NAME] 'Specifies a config name, required only in non-interactive mode.'")
    )
    .subcommand(SubCommand::with_name("upgrade")
      .about("Deploys your repository to an existing App hosted in one of the Deploy Configs from anycloud.json")
      .arg_from_usage("-e, --env-file=[ENV_FILE] 'Specifies an optional environment file relative path'")
      .arg_from_usage("[NON_INTERACTIVE] -n, --non-interactive 'Enables non-interactive CLI mode useful for scripting.'")
      .arg_from_usage("-a, --app-name=[APP_NAME] 'Specifies an optional app name.'")
      .arg_from_usage("-c, --config-name=[CONFIG_NAME] 'Specifies a config name, required only in non-interactive mode.'")
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

  let matches = app.get_matches();
  match matches.subcommand() {
    ("new", Some(matches)) => {
      let non_interactive: bool = matches.values_of("NON_INTERACTIVE").is_some();
      authenticate(non_interactive).await;
      new_or_upgrade(
        anycloud_agz,
        matches.value_of("env-file"),
        matches.value_of("app-name"),
        matches.value_of("config-name"),
        non_interactive,
        |anycloud_agz, anycloud_params, env_b64, app_name, config_name, non_interactive| async move {
          deploy::new(
            anycloud_agz,
            anycloud_params,
            env_b64,
            app_name,
            config_name,
            non_interactive,
          )
          .await
        },
      )
      .await;
    }
    ("terminate", Some(matches)) => {
      let non_interactive: bool = matches.values_of("NON_INTERACTIVE").is_some();
      authenticate(non_interactive).await;
      let app_name = matches.value_of("app-name").map(String::from);
      let config_name = matches.value_of("config-name").map(String::from);
      deploy::terminate(app_name, config_name, non_interactive).await
    }
    ("upgrade", Some(matches)) => {
      let non_interactive: bool = matches.values_of("NON_INTERACTIVE").is_some();
      authenticate(non_interactive).await;
      new_or_upgrade(
        anycloud_agz,
        matches.value_of("env-file"),
        matches.value_of("app-name"),
        matches.value_of("config-name"),
        non_interactive,
        |anycloud_agz, anycloud_params, env_b64, app_name, config_name, non_interactive| async move {
          deploy::upgrade(
            anycloud_agz,
            anycloud_params,
            env_b64,
            app_name,
            config_name,
            non_interactive,
          )
          .await
        },
      )
      .await;
    }
    ("list", _) => {
      authenticate(false).await;
      deploy::info().await
    }
    ("credentials", Some(sub_matches)) => {
      authenticate(false).await;
      match sub_matches.subcommand() {
        ("new", _) => {
          deploy::add_cred(None).await;
        }
        ("edit", _) => deploy::edit_cred().await,
        ("list", _) => deploy::list_creds().await,
        ("remove", _) => deploy::remove_cred().await,
        // rely on AppSettings::SubcommandRequiredElseHelp
        _ => {}
      }
    }
    ("config", Some(sub_matches)) => {
      authenticate(false).await;
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
    _ => {
      authenticate(false).await;
    }
  }
}

async fn new_or_upgrade<Callback, CallbackFut>(
  anycloud_agz: String,
  env_file: Option<&str>,
  app_name: Option<&str>,
  config_name: Option<&str>,
  non_interactive: bool,
  deploy_fn: Callback,
) where
  Callback: Fn(
    String,
    Option<(String, String)>,
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
  ) -> CallbackFut,
  CallbackFut: Future<Output = ()>,
{
  let dockerfile_b64 = get_dockerfile_b64().await;
  let app_tar_gz_b64 = get_app_tar_gz_b64(true).await;
  let env_b64 = match env_file {
    Some(env_file) => Some(get_env_file_b64(env_file.to_string()).await),
    None => None,
  };
  let app_name = app_name.map(String::from);
  let config_name = config_name.map(String::from);
  deploy_fn(
    anycloud_agz,
    Some((dockerfile_b64, app_tar_gz_b64)),
    env_b64,
    app_name,
    config_name,
    non_interactive,
  )
  .await
}
