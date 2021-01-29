use hyper::{client::Client, Body, Request};
use serde::{Deserialize, Serialize};
use serde_json::{from_reader, from_str, json, Value};

use std::error::Error;
use std::fmt::Display;
use std::fs::{File, read};
use std::io::BufReader;
use std::path::Path;

use ascii_table::{AsciiTable, Column, Align};
use base64;
use tokio::runtime::Runtime;

const URL: &str = "https://deploy.alantechnologies.com";

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Serialize)]
struct AWSCredentials {
  accessKeyId: String,
  secretAccessKey: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct AWSConfig {
  credentials: AWSCredentials,
  region: String,
}

#[derive(Deserialize, Debug, Serialize)]
struct Config {
  aws: AWSConfig,
}

#[derive(Deserialize, Debug)]
struct App {
  id: String,
  url: String,
  status: String,
}

const CONFIG_NAME: &str = ".alan/deploy.json";
const CONFIG_SCHEMA: &str = "Please define a deploy config with the following schema: \n{
  \"aws\": {
      \"region\": \"string\",
      \"credentials\": {
        \"accessKeyId\": \"string\",
        \"secretAccessKey\": \"string\",
      }
  }
}";
const HOW_TO_AWS: &str = "
To create an AWS access key follow this tutorial:\n\nhttps://aws.amazon.com/premiumsupport/knowledge-center/create-access-key/\n
Then enable programmatic access for the IAM user, and attach the built-in 'AdministratorAccess' policy to your IAM user.
";

fn get_config() -> Config {
  let home = std::env::var("HOME").unwrap();
  let file_name = &format!("{}/{}", home, CONFIG_NAME);
  let path = Path::new(file_name);
  let file = File::open(path);
  if let Err(err) = file {
    println!("Cannot access deploy config at {}. Error: {}", file_name, err);
    println!("{}", CONFIG_SCHEMA);
    println!("{}", HOW_TO_AWS);
    std::process::exit(1);
  }
  let reader = BufReader::new(file.unwrap());
  let config = from_reader(reader);
  if let Err(err) = config {
    println!("Invalid deploy config. Error: {}", err);
    println!("{}", CONFIG_SCHEMA);
    println!("{}", HOW_TO_AWS);
    std::process::exit(1);
  }
  config.unwrap()
}

fn post(url: String, body: Value) -> Result<String, Box<dyn Error>> {
  let client = Client::builder().build::<_, Body>(hyper_tls::HttpsConnector::new());
  let rt  = Runtime::new().unwrap();
  rt.block_on(async move {
    let req = Request::post(url)
      .header("Content-Type", "application/json")
      .body(body.to_string().into())?;
    let mut resp = client.request(req).await?;
    let data = hyper::body::to_bytes(resp.body_mut()).await?;
    let str = String::from_utf8(data.to_vec())?;
    Ok(str)
  })
}

fn get_app_str(agz_file: &str) -> String {
  let path = Path::new(agz_file);
  if path.extension().is_none() || path.extension().unwrap() != "agz" {
    println!("Deploy failed. The provided file must be an .agz file");
    std::process::exit(1);
  }
  let app = read(agz_file).expect(&format!("Deploy failed parsing {}", agz_file));
  return base64::encode(app);
}

pub fn kill(app_id: &str) {
  let body = json!({
    "deployConfig": get_config(),
    "appId": app_id,
  });
  let resp = post(format!("{}/v1/kill", URL), body);
  match resp {
    Ok(_) => {
      println!("Killing app with id {} if it exists...\n", app_id);
      status();
    }
    Err(err) => {
      println!("Killing app with id {} failed. Error: {}", app_id, err);
    }
  }
}

pub fn new(agz_file: &str) {
  let app_str = get_app_str(agz_file);
  let body = json!({
    "deployConfig": get_config(),
    "agzB64": app_str,
  });
  let body = json!(body);
  let resp = post(format!("{}/v1/new", URL), body);
  match resp {
    Ok(appId) => {
      println!("Creating new app with id {}...\n", appId);
      status();
    }
    Err(err) => {
      println!("Failed to create a new app. Error: {}", err);
    }
  }
}

pub fn upgrade(app_id: &str, agz_file: &str) {
  let app_str = get_app_str(agz_file);
  let body = json!({
    "deployConfig": get_config(),
    "appId": app_id,
    "agzB64": app_str,
  });
  let resp = post(format!("{}/v1/upgrade", URL), body);
  match resp {
    Ok(_) => {
      println!("Upgrading app {}...\n", app_id);
      status();
    }
    Err(err) => {
      println!("Failed to upgrade app {}. Error: {}", app_id, err);
    }
  }
}

pub fn status() {
  let body = json!({
    "deployConfig": get_config(),
  });
  let resp = post(format!("{}/v1/status", URL), body).unwrap();
  let mut apps: Vec<App> = from_str(resp.as_str()).unwrap();

  if apps.len() == 0 {
    println!("No apps deployed using the cloud credentials in {}", CONFIG_NAME);
    return;
  }

  let mut ascii_table = AsciiTable::default();
  ascii_table.max_width = 100;

  let mut column = Column::default();
  column.header = "App Id".into();
  column.align = Align::Left;
  ascii_table.columns.insert(0, column);

  let mut column = Column::default();
  column.header = "Status".into();
  column.align = Align::Center;
  ascii_table.columns.insert(1, column);

  let mut column = Column::default();
  column.header = "Url".into();
  column.align = Align::Right;
  ascii_table.columns.insert(2, column);

  let mut data: Vec<Vec<&dyn Display>> = vec![];
  for app in &mut apps {
    data.push(vec![&app.id, &app.status, &app.url]);
  }

  println!("Status of all apps deployed using the cloud credentials in ~/{}\n", CONFIG_NAME);
  ascii_table.print(data);
}
