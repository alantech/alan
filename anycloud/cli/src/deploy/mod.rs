use dialoguer::{console::style, theme::ColorfulTheme, Confirm, Input, Select};
use hyper::{Request, StatusCode};
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use serde_ini;
use serde_json::{json, Value};

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};

use ascii_table::{AsciiTable, Column};

use crate::http::CLIENT;
use crate::logger::ErrorType;
use crate::oauth::{clear_token, get_token};
use crate::CLUSTER_ID;

pub const ALAN_VERSION: &'static str = env!("ALAN_VERSION");
const REQUEST_TIMEOUT: &str =
  "Operation is still in progress. It might take a few more minutes for \
  the cloud provider to finish up.";
const FORBIDDEN_OPERATION: &str =
  "Please review your credentials. Make sure you have follow all the \
  configuration steps: https://docs.anycloudapp.com/";
const NAME_CONFLICT: &str = "Another application with same App ID already exists.";
const UNAUTHORIZED_OPERATION: &str =
  "Invalid AnyCloud authentication credentials. Please retry and you will be asked to reauthenticate.";

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct AWSCLICredentialsFile {
  default: AWSCLICredentials,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct AWSCLICredentials {
  aws_access_key_id: String,
  aws_secret_access_key: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct AWSCredentials {
  accessKeyId: String,
  secretAccessKey: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct GCPCredentials {
  privateKey: String,
  clientEmail: String,
  projectId: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct AzureCredentials {
  applicationId: String,
  secret: String,
  subscriptionId: String,
  directoryId: String,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum CloudCredentials {
  GCP(GCPCredentials),
  AWS(AWSCredentials),
  Azure(AzureCredentials),
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Serialize)]
pub struct Credentials {
  credentials: CloudCredentials,
  cloudProvider: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Serialize)]
pub struct DeployConfig {
  credentialsName: String,
  region: String,
  vmType: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Serialize)]
pub struct Config {
  credentials: CloudCredentials,
  region: String,
  cloudProvider: String,
  vmType: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
struct App {
  id: String,
  url: String,
  deployName: String,
  status: String,
  size: usize,
  cloudConfigs: Vec<Config>,
}

#[derive(Debug)]
pub enum PostV1Error {
  Timeout,
  Forbidden,
  Conflict,
  Unauthorized,
  Other(String),
}

const ANYCLOUD_FILE: &str = "anycloud.json";
const CREDENTIALS_FILE: &str = ".anycloud/credentials.json";

fn get_aws_cli_creds() -> Result<AWSCLICredentialsFile, String> {
  let home = std::env::var("HOME").unwrap();
  let file_name = &format!("{}/.aws/credentials", home);
  let file = OpenOptions::new().read(true).open(file_name);
  if let Err(err) = file {
    return Err(err.to_string());
  }
  let reader = BufReader::new(file.unwrap());
  match serde_ini::from_bufread(reader) {
    Ok(creds) => Ok(creds),
    Err(err) => Err(err.to_string()),
  }
}

pub async fn add_cred() -> String {
  let mut credentials = get_creds().await;
  let clouds = vec!["AWS", "GCP", "Azure"];
  let selection = Select::with_theme(&ColorfulTheme::default())
    .with_prompt("Pick cloud provider for the new Credential")
    .items(&clouds)
    .default(0)
    .interact()
    .unwrap();
  let cloud = clouds[selection];
  let cred_name = Input::with_theme(&ColorfulTheme::default())
    .with_prompt("Name for new Credential")
    .validate_with(|input: &String| -> Result<(), &str> {
      if credentials.contains_key(input) {
        Err("Credential name already exists")
      } else {
        Ok(())
      }
    })
    .default(cloud.to_lowercase())
    .interact_text()
    .unwrap();
  let name = cred_name.to_string();
  match cloud {
    "AWS" => {
      let aws_cli_creds = get_aws_cli_creds();
      let (access_key, secret) = if aws_cli_creds.is_ok()
        && Confirm::with_theme(&ColorfulTheme::default())
          .with_prompt("Default AWS CLI credentials found. Do you wish to use those?")
          .default(true)
          .interact()
          .unwrap()
      {
        let creds = aws_cli_creds.unwrap().default;
        (creds.aws_access_key_id, creds.aws_secret_access_key)
      } else {
        let access_key: String = Input::with_theme(&ColorfulTheme::default())
          .with_prompt("AWS Access Key ID")
          .interact_text()
          .unwrap();
        let secret: String = Input::with_theme(&ColorfulTheme::default())
          .with_prompt("AWS Secret Access Key")
          .interact_text()
          .unwrap();
        (access_key, secret)
      };
      credentials.insert(
        cred_name,
        Credentials {
          credentials: CloudCredentials::AWS(AWSCredentials {
            accessKeyId: access_key,
            secretAccessKey: secret,
          }),
          cloudProvider: "AWS".to_owned(),
        },
      );
    }
    "GCP" => {
      let project_id: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("GCP Project ID")
        .interact_text()
        .unwrap();
      let client_email: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("GCP Client Email")
        .interact_text()
        .unwrap();
      let private_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("GCP Private Key")
        .interact_text()
        .unwrap();
      credentials.insert(
        cred_name,
        Credentials {
          credentials: CloudCredentials::GCP(GCPCredentials {
            privateKey: private_key,
            clientEmail: client_email,
            projectId: project_id,
          }),
          cloudProvider: "GCP".to_owned(),
        },
      );
    }
    "Azure" => {
      let application_id: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure Application ID")
        .interact_text()
        .unwrap();
      let directory_id: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure Directory ID")
        .interact_text()
        .unwrap();
      let subscription_id: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure Subscription ID")
        .interact_text()
        .unwrap();
      let secret: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure Secret")
        .interact_text()
        .unwrap();
      credentials.insert(
        cred_name,
        Credentials {
          credentials: CloudCredentials::Azure(AzureCredentials {
            applicationId: application_id,
            subscriptionId: subscription_id,
            directoryId: directory_id,
            secret: secret,
          }),
          cloudProvider: "Azure".to_owned(),
        },
      );
    }
    _ => {}
  }
  update_cred_file(credentials).await;
  println!("Successfully created {} Credential", style(&name).bold());
  name
}

async fn update_cred_file(credentials: HashMap<String, Credentials>) {
  let home = std::env::var("HOME").unwrap();
  let file_name = &format!("{}/{}", home, CREDENTIALS_FILE);
  // Sets the option to create a new file, or open it if it already exists.
  // Sets the option for truncating a previous file.
  let file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(file_name);
  let writer = BufWriter::new(file.unwrap());
  if let Err(err) = serde_json::to_writer_pretty(writer, &credentials) {
    error!(
      ErrorType::InvalidCredentialsFile,
      "Failed to write to {}. Error: {}", CREDENTIALS_FILE, err
    )
    .await;
    std::process::exit(1);
  }
}

async fn update_anycloud_file(deploy_configs: HashMap<String, Vec<DeployConfig>>) {
  let home = std::env::var("PWD").unwrap();
  let file_name = &format!("{}/{}", home, ANYCLOUD_FILE);
  // Sets the option to create a new file, or open it if it already exists.
  // Sets the option for truncating a previous file.
  let file = OpenOptions::new()
    .write(true)
    .create(true)
    .truncate(true)
    .open(file_name);
  let writer = BufWriter::new(file.unwrap());
  if let Err(err) = serde_json::to_writer_pretty(writer, &deploy_configs) {
    error!(
      ErrorType::InvalidAnycloudFile,
      "Failed to write to {}. Error: {}", ANYCLOUD_FILE, err
    )
    .await;
    std::process::exit(1);
  }
}

pub async fn edit_cred() {
  let mut credentials = get_creds().await;
  let cred_options = credentials.keys().cloned().collect::<Vec<String>>();
  if cred_options.len() == 0 {
    prompt_add_cred(true).await;
  }
  let selection = Select::with_theme(&ColorfulTheme::default())
    .items(&cred_options)
    .with_prompt("Pick Credentials to edit")
    .default(0)
    .interact()
    .unwrap();
  let name = &cred_options[selection];
  let cred = credentials.get(name).unwrap();
  let cred_name = name.to_string();
  match &cred.credentials {
    CloudCredentials::AWS(cred) => {
      let access_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("AWS Access Key ID")
        .with_initial_text(cred.accessKeyId.to_string())
        .interact_text()
        .unwrap();
      let secret: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("AWS Secret Access Key")
        .with_initial_text(cred.secretAccessKey.to_string())
        .interact_text()
        .unwrap();
      credentials.insert(
        cred_name,
        Credentials {
          credentials: CloudCredentials::AWS(AWSCredentials {
            accessKeyId: access_key,
            secretAccessKey: secret,
          }),
          cloudProvider: "AWS".to_owned(),
        },
      );
    }
    CloudCredentials::GCP(cred) => {
      let client_email: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("GCP Client Email")
        .with_initial_text(cred.clientEmail.to_string())
        .interact_text()
        .unwrap();
      let project_id: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("GCP Project ID")
        .with_initial_text(cred.projectId.to_string())
        .interact_text()
        .unwrap();
      let private_key: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("GCP Private Key")
        .with_initial_text(cred.privateKey.to_string())
        .interact_text()
        .unwrap();
      credentials.insert(
        cred_name,
        Credentials {
          credentials: CloudCredentials::GCP(GCPCredentials {
            privateKey: private_key,
            clientEmail: client_email,
            projectId: project_id,
          }),
          cloudProvider: "GCP".to_owned(),
        },
      );
    }
    CloudCredentials::Azure(cred) => {
      let application_id: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure Application ID")
        .with_initial_text(cred.applicationId.to_string())
        .interact_text()
        .unwrap();
      let directory_id: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure Directory ID")
        .with_initial_text(cred.directoryId.to_owned())
        .interact_text()
        .unwrap();
      let subscription_id: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure Subscription ID")
        .with_initial_text(cred.subscriptionId.to_string())
        .interact_text()
        .unwrap();
      let secret: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Azure Secret")
        .with_initial_text(cred.secret.to_string())
        .interact_text()
        .unwrap();
      credentials.insert(
        cred_name,
        Credentials {
          credentials: CloudCredentials::Azure(AzureCredentials {
            applicationId: application_id,
            subscriptionId: subscription_id,
            directoryId: directory_id,
            secret: secret,
          }),
          cloudProvider: "Azure".to_owned(),
        },
      );
    }
  }
  update_cred_file(credentials).await;
  println!("Successfully edited {} Credentials", style(name).bold());
}

// prompt the user to create a deploy credentials if none exists
pub async fn prompt_add_cred(exit_on_done: bool) -> String {
  let prompt = "No Credentials have been created. Let's create one?";
  if Confirm::with_theme(&ColorfulTheme::default())
    .with_prompt(prompt)
    .default(true)
    .interact()
    .unwrap()
  {
    let cred = add_cred().await;
    if exit_on_done {
      std::process::exit(0)
    }
    cred
  } else {
    std::process::exit(0);
  }
}

// prompt the user to create a deploy config if none exists
pub async fn prompt_add_config() {
  let prompt = "No Deploy Configs have been created. Let's create one?";
  if Confirm::with_theme(&ColorfulTheme::default())
    .with_prompt(prompt)
    .default(true)
    .interact()
    .unwrap()
  {
    add_deploy_config().await;
  }
  std::process::exit(0);
}

pub async fn remove_cred() {
  let mut creds = get_creds().await;
  let cred_options = creds.keys().cloned().collect::<Vec<String>>();
  if cred_options.len() == 0 {
    prompt_add_cred(true).await;
  };
  let selection = Select::with_theme(&ColorfulTheme::default())
    .items(&cred_options)
    .with_prompt("Pick Credentials to remove")
    .default(0)
    .interact()
    .unwrap();
  let cred_name = &cred_options[selection];
  creds.remove(cred_name).unwrap();
  update_cred_file(creds).await;
  println!(
    "Successfully removed {} Credentials",
    style(cred_name).bold()
  );
}

pub async fn list_creds() {
  let credentials = get_creds().await;
  if credentials.len() > 0 {
    for (cred_name, cred) in credentials.into_iter() {
      println!("\n{}", cred_name);
      println!("{}", (0..cred_name.len()).map(|_| "-").collect::<String>());
      match cred.credentials {
        CloudCredentials::AWS(credential) => {
          println!("AWS Access Key ID: {}", credential.accessKeyId);
          println!("AWS Secret Access Key: {}", credential.secretAccessKey);
        }
        CloudCredentials::GCP(credential) => {
          println!("GCP Project ID: {}", credential.projectId);
          println!("GCP Client Email: {}", credential.clientEmail);
          println!("GCP Private Key: {}", credential.privateKey);
        }
        CloudCredentials::Azure(credential) => {
          println!("Azure Application ID: {}", credential.applicationId);
          println!("Azure Directory ID: {}", credential.directoryId);
          println!("Azure Subscription ID: {}", credential.subscriptionId);
          println!("Azure Secret: {}", credential.secret);
        }
      }
    }
  } else {
    prompt_add_cred(true).await;
  }
}

pub async fn add_deploy_config() {
  let mut deploy_configs = get_deploy_configs().await;
  let creds = get_creds().await;
  let name: String = Input::with_theme(&ColorfulTheme::default())
    .with_prompt("Name for new Deploy Config")
    .validate_with(|input: &String| -> Result<(), &str> {
      if deploy_configs.contains_key(input) {
        Err("Deploy Config name already exists")
      } else {
        Ok(())
      }
    })
    .default("staging".into())
    .interact_text()
    .unwrap();
  let mut cloud_configs = Vec::new();
  if creds.len() == 0 {
    prompt_add_cred(false).await;
  }
  let mut options = creds.keys().cloned().collect::<Vec<String>>();
  let new_cred_idx = options.len();
  options.push("Create new Credentials".to_string());
  loop {
    let selection = Select::with_theme(&ColorfulTheme::default())
      .items(&options)
      .with_prompt("Pick Credentials to use")
      .default(0)
      .interact()
      .unwrap();
    let cred = if selection == new_cred_idx {
      add_cred().await
    } else {
      options[selection].to_string()
    };
    // TODO validate these fields?
    let region: String = Input::with_theme(&ColorfulTheme::default())
      .with_prompt("Region name")
      .interact_text()
      .unwrap();
    let vm_type: String = Input::with_theme(&ColorfulTheme::default())
      .with_prompt("Virtual machine type")
      .interact_text()
      .unwrap();
    cloud_configs.push(DeployConfig {
      credentialsName: cred,
      vmType: vm_type,
      region,
    });
    let prompt = if creds.len() > 1 {
      "Do you want to add another region or cloud provider to this Deploy Config?"
    } else {
      "Do you want to add another region to this Deploy Config?"
    };
    if !Confirm::with_theme(&ColorfulTheme::default())
      .with_prompt(prompt)
      .default(false)
      .interact()
      .unwrap()
    {
      break;
    }
  }
  deploy_configs.insert(name.to_string(), cloud_configs);
  update_anycloud_file(deploy_configs).await;
  println!("Successfully created {} Deploy Config.", style(name).bold());
}

pub async fn edit_deploy_config() {
  let mut deploy_configs = get_deploy_configs().await;
  let config_names = deploy_configs.keys().cloned().collect::<Vec<String>>();
  if config_names.len() == 0 {
    prompt_add_config().await;
  }
  let selection = Select::with_theme(&ColorfulTheme::default())
    .items(&config_names)
    .with_prompt("Pick Deploy Config to edit")
    .default(0)
    .interact()
    .unwrap();
  let config_name = config_names[selection].to_string();
  let creds = get_creds().await;
  let cloud_configs: &Vec<DeployConfig> = deploy_configs.get(&config_name).unwrap();
  let mut new_cloud_configs = Vec::new();
  let cred_options = creds.keys().cloned().collect::<Vec<String>>();
  for config in cloud_configs {
    let index = cred_options
      .iter()
      .position(|r| r == &config.credentialsName)
      .unwrap();
    let selection = Select::with_theme(&ColorfulTheme::default())
      .items(&cred_options)
      .with_prompt("Pick Credentials to use")
      .default(index)
      .interact()
      .unwrap();
    let cred = cred_options[selection].to_string();
    let region: String = Input::with_theme(&ColorfulTheme::default())
      .with_prompt("Region name")
      .with_initial_text(config.region.to_string())
      .interact_text()
      .unwrap();
    let vm_type: String = Input::with_theme(&ColorfulTheme::default())
      .with_prompt("Virtual machine type")
      .with_initial_text(config.vmType.to_string())
      .interact_text()
      .unwrap();
    new_cloud_configs.push(DeployConfig {
      credentialsName: cred,
      vmType: vm_type,
      region,
    });
  }
  deploy_configs.insert(config_name.to_string(), new_cloud_configs);
  update_anycloud_file(deploy_configs).await;
  println!(
    "Successfully edited {} Deploy Config.",
    style(config_name).bold()
  );
}

pub async fn remove_deploy_config() {
  let mut deploy_configs = get_deploy_configs().await;
  let config_names = deploy_configs.keys().cloned().collect::<Vec<String>>();
  if config_names.len() == 0 {
    prompt_add_config().await;
  }
  let selection = Select::with_theme(&ColorfulTheme::default())
    .items(&config_names)
    .with_prompt("Pick Deploy Config to remove")
    .default(0)
    .interact()
    .unwrap();
  let config_name = config_names[selection].to_string();
  deploy_configs.remove(&config_name);
  update_anycloud_file(deploy_configs).await;
  println!(
    "Successfully removed {} Deploy Config.",
    style(config_name).bold()
  );
}

pub async fn list_deploy_configs() {
  let mut table = AsciiTable::default();
  table.max_width = 140;
  let configs = get_deploy_configs().await;
  if configs.len() == 0 {
    prompt_add_config().await;
  }
  let mut data: Vec<Vec<&dyn Display>> = vec![];
  for (name, config) in &mut configs.iter() {
    for (i, c) in config.iter().enumerate() {
      if i == 0 {
        data.push(vec![name, &c.credentialsName, &c.region, &c.vmType])
      } else {
        data.push(vec![&"", &c.credentialsName, &c.region, &c.vmType])
      };
    }
  }

  let column = Column {
    header: "Name".into(),
    ..Column::default()
  };
  table.columns.insert(0, column);

  let column = Column {
    header: "Credentials Name".into(),
    ..Column::default()
  };
  table.columns.insert(1, column);

  let column = Column {
    header: "Region".into(),
    ..Column::default()
  };
  table.columns.insert(2, column);

  let column = Column {
    header: "VM Type".into(),
    ..Column::default()
  };
  table.columns.insert(3, column);

  if configs.len() > 0 {
    println!("\nDeployment configurations:\n");
    table.print(data);
  } else {
    println!("No Deploy Configs to display from anycloud.json.")
  }
}

async fn get_creds() -> HashMap<String, Credentials> {
  let home = std::env::var("HOME").unwrap();
  let file_name = &format!("{}/{}", home, CREDENTIALS_FILE);
  let file = OpenOptions::new().read(true).open(file_name);
  if let Err(_) = file {
    return HashMap::new();
  }
  let reader = BufReader::new(file.unwrap());
  let creds = serde_json::from_reader(reader);
  if let Err(err) = creds {
    error!(
      ErrorType::InvalidCredentialsFile,
      "Failed to read from {}. Error: {}", CREDENTIALS_FILE, err
    )
    .await;
    std::process::exit(1);
  }
  creds.unwrap()
}

async fn get_deploy_configs() -> HashMap<String, Vec<DeployConfig>> {
  let home = std::env::var("PWD").unwrap();
  let file_name = &format!("{}/{}", home, ANYCLOUD_FILE);
  let file = OpenOptions::new().read(true).open(file_name);
  if let Err(_) = file {
    return HashMap::new();
  }
  let reader = BufReader::new(file.unwrap());
  let config = serde_json::from_reader(reader);
  if let Err(err) = config {
    error!(
      ErrorType::InvalidAnycloudFile,
      "Failed to read from {}. Error: {}", ANYCLOUD_FILE, err
    )
    .await;
    std::process::exit(1);
  }
  config.unwrap()
}

// This method can be called as a binary by the end user in the CLI or as a library by the Alan daemon
// to send stats to the deploy service. We default to production so that it works as-is when it is used
// as a binary and we override the value it can have to our needs
fn get_url() -> &'static str {
  let env = std::env::var("ALAN_TECH_ENV").unwrap_or("production".to_string());
  match env.as_str() {
    "local" => "http://localhost:8080",
    "staging" => "https://deploy-staging.alantechnologies.com",
    _ => "https://deploy.alantechnologies.com",
  }
}

pub async fn get_config() -> HashMap<String, Vec<Config>> {
  let anycloud_prof = get_deploy_configs().await;
  let creds = get_creds().await;
  if creds.len() == 0 {
    prompt_add_cred(true).await;
  }
  if anycloud_prof.len() == 0 {
    prompt_add_config().await;
  }
  let mut all_configs = HashMap::new();
  for (deploy_name, deploy_configs) in anycloud_prof.into_iter() {
    let mut configs = Vec::new();
    for deploy_config in deploy_configs {
      let cred = creds.get(&deploy_config.credentialsName).unwrap();
      configs.push(Config {
        credentials: cred.credentials.clone(),
        cloudProvider: cred.cloudProvider.to_string(),
        region: deploy_config.region,
        vmType: deploy_config.vmType,
      });
    }
    all_configs.insert(deploy_name, configs);
  }
  all_configs
}

pub async fn post_v1(endpoint: &str, body: Value) -> Result<String, PostV1Error> {
  let url = get_url();
  let req = Request::post(format!("{}/v1/{}", url, endpoint))
    .header("Content-Type", "application/json")
    .body(body.to_string().into());
  let req = match req {
    Ok(req) => req,
    Err(e) => return Err(PostV1Error::Other(e.to_string())),
  };
  let resp = CLIENT.request(req).await;
  let mut resp = match resp {
    Ok(resp) => resp,
    Err(e) => return Err(PostV1Error::Other(e.to_string())),
  };
  let data = hyper::body::to_bytes(resp.body_mut()).await;
  let data = match data {
    Ok(data) => data,
    Err(e) => return Err(PostV1Error::Other(e.to_string())),
  };
  let data_str = String::from_utf8(data.to_vec());
  let data_str = match data_str {
    Ok(data_str) => data_str,
    Err(e) => return Err(PostV1Error::Other(e.to_string())),
  };
  return match resp.status() {
    st if st.is_success() => Ok(data_str),
    StatusCode::REQUEST_TIMEOUT => Err(PostV1Error::Timeout),
    StatusCode::FORBIDDEN => Err(PostV1Error::Forbidden),
    StatusCode::CONFLICT => Err(PostV1Error::Conflict),
    _ => Err(PostV1Error::Other(data_str.to_string())),
  };
}

pub async fn client_error(err_code: ErrorType, message: &str) {
  let mut body = json!({
    "errorCode": err_code as u64,
    "accessToken": get_token(),
    "alanVersion": format!("v{}", ALAN_VERSION),
    "osName": std::env::consts::OS,
    "message": message,
  });
  if let Some(cluster_id) = CLUSTER_ID.get() {
    body
      .as_object_mut()
      .unwrap()
      .insert(format!("clusterId"), json!(cluster_id));
  }
  let _resp = post_v1("clientError", body).await;
}

pub async fn terminate() {
  let apps = get_apps(false).await;
  let ids = apps.iter().map(|a| a.id.as_str()).collect::<Vec<&str>>();
  let selection = Select::with_theme(&ColorfulTheme::default())
    .items(&ids)
    .with_prompt("Pick App to terminate")
    .default(0)
    .interact()
    .unwrap();
  let cluster_id = &ids[selection];
  CLUSTER_ID.set(cluster_id.to_string()).unwrap();
  let styled_cluster_id = style(cluster_id).bold();
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message(&format!("Terminating App {}", styled_cluster_id));
  let body = json!({
    "deployConfig": get_config().await,
    "clusterId": cluster_id,
    "accessToken": get_token(),
  });
  let resp = post_v1("terminate", body).await;
  let res = match resp {
    Ok(_) => format!("Terminated App {} successfully!", styled_cluster_id),
    Err(err) => match err {
      PostV1Error::Timeout => format!("{}", REQUEST_TIMEOUT),
      PostV1Error::Forbidden => format!("{}", FORBIDDEN_OPERATION),
      PostV1Error::Conflict => format!(
        "Failed to terminate App {}. Error: {}",
        cluster_id, NAME_CONFLICT
      ),
      PostV1Error::Unauthorized => {
        clear_token();
        format!("{}", UNAUTHORIZED_OPERATION)
      }
      PostV1Error::Other(err) => format!(
        "Failed to terminate App {}. Error: {}",
        styled_cluster_id, err
      ),
    },
  };
  sp.finish_with_message(&res);
}

pub async fn new(
  agz_b64: String,
  anycloud_params: Option<(String, String)>,
  env_b64: Option<String>,
) {
  let config = get_config().await;
  let config_names = config.keys().cloned().collect::<Vec<String>>();
  if config_names.len() == 0 {
    prompt_add_config().await;
  }
  let selection = Select::with_theme(&ColorfulTheme::default())
    .items(&config_names)
    .with_prompt("Pick Deploy Config for App")
    .default(0)
    .interact()
    .unwrap();
  let deploy_config = &config_names[selection];
  let app_id: std::io::Result<String> = Input::with_theme(&ColorfulTheme::default())
    .with_prompt("Optional App name")
    .allow_empty(true)
    .interact_text();
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message("Creating new App");
  let mut body = json!({
    "deployName": deploy_config,
    "deployConfig": config,
    "agzB64": agz_b64,
    "alanVersion": format!("v{}", ALAN_VERSION),
    "accessToken": get_token(),
    "osName": std::env::consts::OS,
  });
  let mut_body = body.as_object_mut().unwrap();
  if let Ok(app_id) = app_id {
    mut_body.insert(format!("appId"), json!(app_id));
  }
  if let Some(anycloud_params) = anycloud_params {
    mut_body.insert(format!("DockerfileB64"), json!(anycloud_params.0));
    mut_body.insert(format!("appTarGzB64"), json!(anycloud_params.1));
  }
  if let Some(env_b64) = env_b64 {
    mut_body.insert(format!("envB64"), json!(env_b64));
  }
  let resp = post_v1("new", body).await;
  let res = match resp {
    Ok(res) => format!("Created App {} successfully!", style(res).bold()),
    Err(err) => match err {
      PostV1Error::Timeout => format!("{}", REQUEST_TIMEOUT),
      PostV1Error::Forbidden => format!("{}", FORBIDDEN_OPERATION),
      PostV1Error::Conflict => format!("Failed to create a new App. Error: {}", NAME_CONFLICT),
      PostV1Error::Unauthorized => {
        clear_token();
        format!("{}", UNAUTHORIZED_OPERATION)
      }
      PostV1Error::Other(err) => format!("Failed to create a new App. Error: {}", err),
    },
  };
  sp.finish_with_message(&res);
}

pub async fn upgrade(
  agz_b64: String,
  anycloud_params: Option<(String, String)>,
  env_b64: Option<String>,
) {
  let apps = get_apps(false).await;
  let ids = apps.iter().map(|a| a.id.as_str()).collect::<Vec<&str>>();
  let selection = Select::with_theme(&ColorfulTheme::default())
    .items(&ids)
    .with_prompt("Pick App to upgrade")
    .default(0)
    .interact()
    .unwrap();
  let cluster_id = ids[selection];
  CLUSTER_ID.set(cluster_id.to_string()).unwrap();
  let styled_cluster_id = style(cluster_id).bold();
  let config = get_config().await;
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message(&format!("Upgrading App {}", styled_cluster_id));
  let mut body = json!({
    "clusterId": cluster_id,
    "deployConfig": config,
    "agzB64": agz_b64,
    "alanVersion": format!("v{}", ALAN_VERSION),
    "accessToken": get_token(),
    "osName": std::env::consts::OS,
  });
  let mut_body = body.as_object_mut().unwrap();
  if let Some(anycloud_params) = anycloud_params {
    mut_body.insert(format!("DockerfileB64"), json!(anycloud_params.0));
    mut_body.insert(format!("appTarGzB64"), json!(anycloud_params.1));
  }
  if let Some(env_b64) = env_b64 {
    mut_body.insert(format!("envB64"), json!(env_b64));
  }
  let resp = post_v1("upgrade", body).await;
  let res = match resp {
    Ok(_) => format!("Upgraded App {} successfully!", styled_cluster_id),
    Err(err) => match err {
      PostV1Error::Timeout => format!("{}", REQUEST_TIMEOUT),
      PostV1Error::Forbidden => format!("{}", FORBIDDEN_OPERATION),
      PostV1Error::Conflict => format!("Failed to create a new app. Error: {}", NAME_CONFLICT),
      PostV1Error::Unauthorized => {
        clear_token();
        format!("{}", UNAUTHORIZED_OPERATION)
      }
      PostV1Error::Other(err) => format!("Failed to create a new app. Error: {}", err),
    },
  };
  sp.finish_with_message(&res);
}

async fn get_apps(status: bool) -> Vec<App> {
  let config = get_config().await;
  let token = get_token();
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message("Gathering information about Apps deployed");
  let body = json!({
    "deployConfig": config,
    "accessToken": token,
    "status": status,
  });
  let response = post_v1("info", body).await;
  sp.finish_and_clear();
  let resp = match &response {
    Ok(resp) => resp,
    Err(err) => {
      match err {
        PostV1Error::Timeout => {
          eprintln!("{}", REQUEST_TIMEOUT);
        }
        PostV1Error::Forbidden => {
          eprintln!("{}", FORBIDDEN_OPERATION);
        }
        PostV1Error::Conflict => {
          eprintln!(
            "Displaying status for Apps failed with error: {}",
            NAME_CONFLICT
          );
        }
        PostV1Error::Unauthorized => {
          clear_token();
          eprintln!("{}", UNAUTHORIZED_OPERATION);
        }
        PostV1Error::Other(err) => {
          eprintln!("Displaying status for Apps failed with error: {}", err);
        }
      }
      std::process::exit(1);
    }
  };
  let apps: Vec<App> = serde_json::from_str(resp).unwrap();
  if apps.len() == 0 {
    println!("No Apps currently deployed");
    std::process::exit(0);
  }
  apps
}

pub async fn info() {
  let mut apps = get_apps(true).await;

  let mut clusters = AsciiTable::default();
  clusters.max_width = 140;

  let column = Column {
    header: "App ID".into(),
    ..Column::default()
  };
  clusters.columns.insert(0, column);

  let column = Column {
    header: "Url".into(),
    ..Column::default()
  };
  clusters.columns.insert(1, column);

  let column = Column {
    header: "Deploy Config".into(),
    ..Column::default()
  };
  clusters.columns.insert(2, column);

  let column = Column {
    header: "Size".into(),
    ..Column::default()
  };
  clusters.columns.insert(3, column);

  let column = Column {
    header: "Status".into(),
    ..Column::default()
  };
  clusters.columns.insert(4, column);

  let mut app_data: Vec<Vec<&dyn Display>> = vec![];
  let mut profile_data: Vec<Vec<&dyn Display>> = vec![];
  let mut deploy_profiles = HashSet::new();
  for app in &mut apps {
    app_data.push(vec![
      &app.id,
      &app.url,
      &app.deployName,
      &app.size,
      &app.status,
    ]);
    if deploy_profiles.contains(&app.deployName) {
      continue;
    }
    for (i, profile) in app.cloudConfigs.iter().enumerate() {
      if i == 0 {
        profile_data.push(vec![&app.deployName, &profile.region, &profile.vmType])
      } else {
        profile_data.push(vec![&"", &profile.region, &profile.vmType])
      };
    }
    deploy_profiles.insert(&app.deployName);
  }

  println!("Apps deployed:\n");
  clusters.print(app_data);

  let mut profiles = AsciiTable::default();
  profiles.max_width = 140;

  let column = Column {
    header: "Deploy Config".into(),
    ..Column::default()
  };
  profiles.columns.insert(0, column);

  let column = Column {
    header: "Region".into(),
    ..Column::default()
  };
  profiles.columns.insert(1, column);

  let column = Column {
    header: "VM Type".into(),
    ..Column::default()
  };
  profiles.columns.insert(2, column);
  println!("\nDeploy Configs used:\n");
  profiles.print(profile_data);
}
