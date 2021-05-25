use dialoguer::console::style;
use hyper::{Request, StatusCode};
use indicatif::ProgressBar;
use serde::{Deserialize, Serialize};
use serde_ini;
use serde_json::{json, Value};

use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};
use std::time::Duration;

use ascii_table::{AsciiTable, Column};

use crate::http::CLIENT;
use crate::logger::ErrorType;
use crate::oauth::{clear_token, get_token};
use crate::CLUSTER_ID;

mod anycloud_dialoguer;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const REQUEST_TIMEOUT: &str =
  "Operation is still in progress. It might take a few more minutes for \
  the cloud provider to finish up.";
const FORBIDDEN_OPERATION: &str =
  "Please review your credentials. Make sure you have follow all the \
  configuration steps: https://docs.anycloudapp.com/";
const NAME_CONFLICT: &str = "Another application with same App ID already exists.";
const UNAUTHORIZED_OPERATION: &str =
  "Invalid AnyCloud authentication credentials. Please retry and you will be asked to reauthenticate.";
const BURSTABLE_VM_TYPES: [&'static str; 43] = [
  // AWS: https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/burstable-performance-instances.html
  "t2.nano",
  "t2.micro",
  "t2.small",
  "t2.medium",
  "t2.large",
  "t2.xlarge",
  "t2.2xlarge",
  "t3.nano",
  "t3.micro",
  "t3.small",
  "t3.medium",
  "t3.large",
  "t3.xlarge",
  "t3.2xlarge",
  "t3a.nano",
  "t3a.micro",
  "t3a.small",
  "t3a.medium",
  "t3a.large",
  "t3a.xlarge",
  "t3a.2xlarge",
  "t4g.nano",
  "t4g.micro",
  "t4g.small",
  "t4g.medium",
  "t4g.large",
  "t4g.xlarge",
  "t4g.2xlarge",
  // GCP: https://cloud.google.com/compute/docs/machine-types#cpu-bursting
  "f1-micro",
  "g1-small",
  "e2-micro",
  "e2-small",
  "e2-medium",
  // Azure: https://docs.microsoft.com/en-us/azure/virtual-machines/sizes-b-series-burstable
  "Standard_B1ls",
  "Standard_B1s",
  "Standard_B1ms",
  "Standard_B2s",
  "Standard_B2ms",
  "Standard_B4ms",
  "Standard_B8ms",
  "Standard_B12ms",
  "Standard_B16ms",
  "Standard_B20ms",
];
// VM types with 1GB of memory or less
// AWS: aws ec2 describe-instance-types --filters Name=memory-info.size-in-mib,Values=512,1024 | jq '.InstanceTypes[] | .InstanceType'
// GCP: gcloud compute machine-types list --filter="memoryMb:(512 1024)" --format json | jq '.[] | .name'
// Azure: az vm list-sizes -l westus | jq '.[] | if .memoryInMb <= 1024 then .name else "" end'
const SMALL_VM_TYPES: [&'static str; 13] = [
  "t4g.nano",
  "t2.micro",
  "t3.micro",
  "t4g.micro",
  "t3.nano",
  "t2.nano",
  "t3a.nano",
  "t3a.micro",
  "e2-micro",
  "Standard_B1ls",
  "Standard_B1s",
  "Standard_A0",
  "Basic_A0",
];

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
  #[serde(skip_serializing_if = "Option::is_none")]
  region: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  vmType: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  minReplicas: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  maxReplicas: Option<u32>,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Serialize)]
pub struct Config {
  credentials: CloudCredentials,
  cloudProvider: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  region: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  vmType: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  minReplicas: Option<u32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  maxReplicas: Option<u32>,
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
  let clouds = vec!["AWS".to_string(), "GCP".to_string(), "Azure".to_string()];
  let selection = anycloud_dialoguer::select_with_default(
    "Pick cloud provider for the new Credential",
    &clouds,
    0,
  );
  let cloud = &clouds[selection];
  let default = cloud.to_lowercase();
  let prompt = "Name for new Credential";
  let validator = |input: &String| -> Result<(), &str> {
    if credentials.contains_key(input) {
      Err("Credential name already exists")
    } else {
      Ok(())
    }
  };
  let cred_name = if credentials.contains_key(&default) {
    anycloud_dialoguer::input_with_validation(prompt, validator)
  } else {
    anycloud_dialoguer::input_with_default_and_validation(prompt, default, validator)
  };
  let name = cred_name.to_string();
  match cloud.as_str() {
    "AWS" => {
      let aws_cli_creds = get_aws_cli_creds();
      let (access_key, secret) = if aws_cli_creds.is_ok()
        && anycloud_dialoguer::confirm_with_default(
          "Default AWS CLI credentials found. Do you wish to use those?",
          true,
        ) {
        let creds = aws_cli_creds.unwrap().default;
        (creds.aws_access_key_id, creds.aws_secret_access_key)
      } else {
        let access_key: String = anycloud_dialoguer::input("AWS Access Key ID");
        let secret: String = anycloud_dialoguer::input("AWS Secret Access Key");
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
      let project_id: String = anycloud_dialoguer::input("GCP Project ID");
      let client_email: String = anycloud_dialoguer::input("GCP Client Email");
      let private_key: String = anycloud_dialoguer::input("GCP Private Key");
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
      let application_id: String = anycloud_dialoguer::input("Azure Application ID");
      let directory_id: String = anycloud_dialoguer::input("Azure Directory ID");
      let subscription_id: String = anycloud_dialoguer::input("Azure Subscription ID");
      let secret: String = anycloud_dialoguer::input("Azure Secret");
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
    warn!(
      InvalidCredentialsFile,
      "Failed to write to {}. Error: {}", CREDENTIALS_FILE, err
    );
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
    warn!(
      InvalidAnycloudFile,
      "Failed to write to {}. Error: {}", ANYCLOUD_FILE, err
    );
    std::process::exit(1);
  }
}

pub async fn edit_cred() {
  let mut credentials = get_creds().await;
  let cred_options = credentials.keys().cloned().collect::<Vec<String>>();
  if cred_options.len() == 0 {
    prompt_add_cred(true).await;
  }
  let selection =
    anycloud_dialoguer::select_with_default("Pick Credentials to edit", &cred_options, 0);
  let name = &cred_options[selection];
  let cred = credentials.get(name).unwrap();
  let cred_name = name.to_string();
  match &cred.credentials {
    CloudCredentials::AWS(cred) => {
      let access_key: String = anycloud_dialoguer::input_with_initial_text(
        "AWS Access Key ID",
        cred.accessKeyId.to_string(),
      );
      let secret: String = anycloud_dialoguer::input_with_initial_text(
        "AWS Secret Access Key",
        cred.secretAccessKey.to_string(),
      );
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
      let client_email: String = anycloud_dialoguer::input_with_initial_text(
        "GCP Client Email",
        cred.clientEmail.to_string(),
      );
      let project_id: String =
        anycloud_dialoguer::input_with_initial_text("GCP Project ID", cred.projectId.to_string());
      let private_key: String =
        anycloud_dialoguer::input_with_initial_text("GCP Private Key", cred.privateKey.to_string());
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
      let application_id: String = anycloud_dialoguer::input_with_initial_text(
        "Azure Application ID",
        cred.applicationId.to_string(),
      );
      let directory_id: String = anycloud_dialoguer::input_with_initial_text(
        "Azure Directory ID",
        cred.directoryId.to_owned(),
      );
      let subscription_id: String = anycloud_dialoguer::input_with_initial_text(
        "Azure Subscription ID",
        cred.subscriptionId.to_string(),
      );
      let secret: String =
        anycloud_dialoguer::input_with_initial_text("Azure Secret", cred.secret.to_string());
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
  if anycloud_dialoguer::confirm_with_default(prompt, true) {
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
  if anycloud_dialoguer::confirm_with_default(prompt, true) {
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
  let selection =
    anycloud_dialoguer::select_with_default("Pick Credentials to remove", &cred_options, 0);
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
  let default = "staging".to_string();
  let prompt = "Name for new Deploy Config";
  let validator = |input: &String| -> Result<(), &str> {
    if deploy_configs.contains_key(input) {
      Err("Deploy Config name already exists")
    } else {
      Ok(())
    }
  };
  let name = if deploy_configs.contains_key(&default) {
    anycloud_dialoguer::input_with_validation(prompt, validator)
  } else {
    anycloud_dialoguer::input_with_default_and_validation(prompt, default, validator)
  };
  let mut cloud_configs = Vec::new();
  if creds.len() == 0 {
    prompt_add_cred(false).await;
  }
  let mut options = creds.keys().cloned().collect::<Vec<String>>();
  let new_cred_idx = options.len();
  options.push("Create new Credentials".to_string());
  loop {
    let selection = anycloud_dialoguer::select_with_default("Pick Credentials to use", &options, 0);
    let cred = if selection == new_cred_idx {
      add_cred().await
    } else {
      options[selection].to_string()
    };
    // TODO validate these fields?
    let mut region = None;
    if anycloud_dialoguer::confirm_with_default(
      "Do you want to choose a specific region for this Deploy Config?",
      false,
    ) {
      let input_region: String = anycloud_dialoguer::input("Region name");
      region = Some(input_region);
    };
    let mut vm_type = None;
    if anycloud_dialoguer::confirm_with_default(
      "Do you want to select which virtual machine type to use for this Deploy Config?",
      false,
    ) {
      vm_type = get_some_vm_type_input();
    };
    cloud_configs.push(DeployConfig {
      credentialsName: cred,
      vmType: vm_type,
      region,
      minReplicas: None,
      maxReplicas: None,
    });
    let prompt = if creds.len() > 1 {
      "Do you want to add another region or cloud provider to this Deploy Config?"
    } else {
      "Do you want to add another region to this Deploy Config?"
    };
    if !anycloud_dialoguer::confirm_with_default(prompt, false) {
      break;
    }
  }
  let prompt = if creds.len() > 1 {
    "Minimum number of VMs per region or cloud"
  } else {
    "Minimum number of VMs per region"
  };
  let replicas: String = anycloud_dialoguer::input_with_default(prompt, "1".to_string());
  let min_replicas: Option<u32> = Some(replicas.parse::<u32>().unwrap_or_else(|_| {
    eprintln!("{} is not a valid number of VMs", replicas);
    std::process::exit(1);
  }));
  let mut max_replicas = None;
  let prompt = "Would you like to define a maximum number of VMs?";
  if anycloud_dialoguer::confirm_with_default(prompt, false) {
    let prompt = if creds.len() > 1 {
      "Maximum number of VMs per region or cloud"
    } else {
      "Maximum number of VMs per region"
    };
    let replicas: String = anycloud_dialoguer::input(prompt);
    if let Ok(replicas) = replicas.parse::<u32>() {
      max_replicas = Some(replicas);
    } else {
      eprintln!("{} is not a valid number of VMs", replicas);
      std::process::exit(1);
    }
  }
  cloud_configs = cloud_configs
    .into_iter()
    .map(|mut c| {
      c.minReplicas = min_replicas;
      c.maxReplicas = max_replicas;
      c
    })
    .collect();
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
  let selection =
    anycloud_dialoguer::select_with_default("Pick Deploy Config to edit", &config_names, 0);
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
    let selection =
      anycloud_dialoguer::select_with_default("Pick Credentials to use", &cred_options, index);
    let cred = cred_options[selection].to_string();
    let mut region = None;
    let mut vm_type = None;
    if let Some(reg) = &config.region {
      if anycloud_dialoguer::confirm_with_default(
        "Do you want to edit the region to use for this Deploy Config?",
        true,
      ) {
        let input_region: String = anycloud_dialoguer::input("Region name");
        region = Some(input_region);
      } else {
        region = Some(reg.to_string());
      }
    } else {
      if anycloud_dialoguer::confirm_with_default(
        "Do you want to choose a specific region for this Deploy Config?",
        false,
      ) {
        let input_region: String = anycloud_dialoguer::input("Region name");
        region = Some(input_region);
      };
    }
    if let Some(vm_t) = &config.vmType {
      if anycloud_dialoguer::confirm_with_default(
        "Do you want to edit the virtual machine type for this Deploy Config?",
        true,
      ) {
        vm_type = get_some_vm_type_input();
      } else {
        vm_type = Some(vm_t.to_string());
      }
    } else {
      if anycloud_dialoguer::confirm_with_default(
        "Do you want to select which virtual machine type to use for this Deploy Config?",
        false,
      ) {
        vm_type = get_some_vm_type_input();
      };
    }
    new_cloud_configs.push(DeployConfig {
      credentialsName: cred,
      vmType: vm_type,
      region,
      minReplicas: None,
      maxReplicas: None,
    });
  }
  let prompt = if creds.len() > 1 {
    "Minimum number of VMs per region or cloud"
  } else {
    "Minimum number of VMs per region"
  };
  let replicas: String = anycloud_dialoguer::input_with_default(prompt, "1".to_string());
  let min_replicas: Option<u32> = Some(replicas.parse::<u32>().unwrap_or_else(|_| {
    eprintln!("{} is not a valid number of VMs", replicas);
    std::process::exit(1);
  }));
  let mut max_replicas = None;
  let prompt = "Would you like to define a maximum number of VMs?";
  if anycloud_dialoguer::confirm_with_default(prompt, false) {
    let prompt = if creds.len() > 1 {
      "Maximum number of VMs per region or cloud"
    } else {
      "Maximum number of VMs per region"
    };
    let replicas: String = anycloud_dialoguer::input(prompt);
    if let Ok(replicas) = replicas.parse::<u32>() {
      max_replicas = Some(replicas);
    } else {
      eprintln!("{} is not a valid number of VMs", replicas);
      std::process::exit(1);
    }
  }
  new_cloud_configs = new_cloud_configs
    .into_iter()
    .map(|mut c| {
      c.minReplicas = min_replicas;
      c.maxReplicas = max_replicas;
      c
    })
    .collect();
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
  let selection =
    anycloud_dialoguer::select_with_default("Pick Deploy Config to remove", &config_names, 0);
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
      let mut display_vec: Vec<&dyn Display> = Vec::new();
      if i == 0 {
        display_vec.push(name);
      } else {
        display_vec.push(&"");
      };
      display_vec.push(&c.credentialsName);
      if let Some(region) = &c.region {
        display_vec.push(region);
      }
      if let Some(vm_type) = &c.vmType {
        display_vec.push(vm_type);
      }
      data.push(display_vec)
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
    warn!(
      InvalidCredentialsFile,
      "Failed to read from {}. Error: {}", CREDENTIALS_FILE, err
    );
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
    warn!(
      InvalidAnycloudFile,
      "Failed to read from {}. Error: {}", ANYCLOUD_FILE, err
    );
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
        minReplicas: deploy_config.minReplicas,
        maxReplicas: deploy_config.maxReplicas,
      });
    }
    all_configs.insert(deploy_name, configs);
  }
  all_configs
}

pub async fn post_v1(endpoint: &str, mut body: Value) -> Result<String, PostV1Error> {
  let url = get_url();
  let mut_body = body.as_object_mut().unwrap();
  mut_body.insert(format!("accessToken"), json!(get_token()));
  mut_body.insert(format!("alanVersion"), json!(format!("v{}", VERSION)));
  mut_body.insert(format!("osName"), json!(std::env::consts::OS));
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

pub async fn client_error(err_code: ErrorType, message: &str, level: &str) {
  let mut body = json!({
    "errorCode": err_code as u64,
    "message": message,
    "level": level,
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
  let ids = apps.into_iter().map(|a| a.id).collect::<Vec<String>>();
  let selection = anycloud_dialoguer::select_with_default("Pick App to terminate", &ids, 0);
  let cluster_id = &ids[selection];
  CLUSTER_ID.set(cluster_id.to_string()).unwrap();
  let styled_cluster_id = style(cluster_id).bold();
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message(&format!("Terminating App {}", styled_cluster_id));
  let body = json!({
    "deployConfig": get_config().await,
    "clusterId": cluster_id,
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
  let selection =
    anycloud_dialoguer::select_with_default("Pick Deploy Config for App", &config_names, 0);
  let deploy_config = &config_names[selection];
  let app_id: std::io::Result<String> =
    anycloud_dialoguer::input_with_allow_empty_as_result("Optional App name", true);
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message("Creating new App");
  let mut body = json!({
    "deployName": deploy_config,
    "deployConfig": config,
    "agzB64": agz_b64,
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
    Ok(res) => {
      // idc if it's been set before, I'm setting it now!!!
      let _ = CLUSTER_ID.set(res);
      poll().await;
      return;
    }
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
  let ids = apps.into_iter().map(|a| a.id).collect::<Vec<String>>();
  let selection = anycloud_dialoguer::select_with_default("Pick App to upgrade", &ids, 0);
  let cluster_id = &ids[selection];
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
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  sp.set_message("Gathering information about Apps deployed");
  let body = json!({
    "deployConfig": config,
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
      let mut display_vec: Vec<&dyn Display> = Vec::new();
      if i == 0 {
        display_vec.push(&app.deployName);
      } else {
        display_vec.push(&"");
      };
      if let Some(region) = &profile.region {
        display_vec.push(region);
      }
      if let Some(vm_type) = &profile.vmType {
        display_vec.push(vm_type);
      }
      profile_data.push(display_vec)
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

pub async fn poll() {
  let mut lines: Vec<String> = vec![];
  let sp = ProgressBar::new_spinner();
  sp.enable_steady_tick(10);
  let logs_url = format!("{}/logs", get_url());
  let body = json!({
    "clusterId": CLUSTER_ID.get().expect("cluster ID not set..."),
  });
  loop {
    let req = Request::get(&logs_url)
      .header("Content-Type", "application/json")
      .body((&body).to_string().into())
      .unwrap();
    let mut res = CLIENT.request(req).await.unwrap();
    let data = hyper::body::to_bytes(res.body_mut()).await.unwrap();
    let data_str = String::from_utf8(data.to_vec()).unwrap();
    // it's ok to leave out the newline chars, since `sp.println` will insert
    // those for us
    let new_lines = data_str.split("\n").skip(lines.len()).collect::<Vec<_>>();
    if new_lines.len() != 0 {
      // update the spinner and lines above the spinner
      new_lines.into_iter().for_each(|line| {
        if let Some(last_line) = lines.get(lines.len() - 1) {
          sp.println(last_line);
        }
        sp.set_message(line);
        lines.push(line.to_string());
      });
      let apps = get_apps(true).await;
      if apps.into_iter().all(|app| app.status == "up") {
        sp.finish();
        break;
      }
    }
    tokio::time::sleep(Duration::from_secs(10)).await;
  }
}

fn is_burstable(vm_type: &str) -> bool {
  BURSTABLE_VM_TYPES.contains(&vm_type)
}

fn is_small(vm_type: &str) -> bool {
  SMALL_VM_TYPES.contains(&vm_type)
}

fn print_vm_type_warns(vm_type: &str) -> () {
  if is_burstable(vm_type) {
    print_burstable_vm_warn();
  }
  if is_small(vm_type) {
    print_small_vm_warn();
  }
}

fn print_burstable_vm_warn() -> () {
  println!(
    "WARNING: You have selected a burstable virtual machine type. \
    These virtual machine types can misbehave under heavy load and \
    do not work correctly with our automatic scale."
  )
}

// Warn if user choose a machine type with 1GB or less memory
fn print_small_vm_warn() -> () {
  println!(
    "WARNING: You have selected a virtual machine type that is too small. \
    These virtual machine types can underperform and take more time to start."
  )
}

fn get_some_vm_type_input() -> Option<String> {
  loop {
    let input_vm_type: String = anycloud_dialoguer::input("Virtual machine type");
    if is_burstable(&input_vm_type) || is_small(&input_vm_type) {
      print_vm_type_warns(&input_vm_type);
      if anycloud_dialoguer::confirm_with_default(
        "Are you sure you want to continue with the selected virtual machine type?",
        false,
      ) {
        return Some(input_vm_type);
      }
    } else {
      return Some(input_vm_type);
    }
  }
}
