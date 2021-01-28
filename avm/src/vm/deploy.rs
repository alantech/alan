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

const URL: &str = "https://alan-deploy-prod.herokuapp.com";

#[derive(Deserialize, Debug, Serialize)]
struct AWSConfig {
  credentials: AWSCredentials,
  region: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Serialize)]
struct AWSCredentials {
  accessKeyId: String,
  secretAccessKey: String,
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

fn get_config() -> Config {
  let home = std::env::var("HOME").unwrap();
  let file_name = &format!("{}/{}", home, CONFIG_NAME);
  let path = Path::new(file_name);
  let file = File::open(path).expect(&format!("{} does not exist. Please define one.", file_name));
  let reader = BufReader::new(file);
  from_reader(reader).expect(&format!("Invalid deployment configuration at {}", file_name))
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
  let res = post(format!("{}/kill", URL), body);
  if res.is_ok() {
    println!("Killing Alan app with id {} if it exists...\n", app_id);
    status();
  } else {
    println!("Killing Alan app with id {} failed.", app_id);
  }
}

pub fn new(agz_file: &str) {
  let app_str = get_app_str(agz_file);
  let body = json!({
    "deployConfig": get_config(),
    "agzB64": app_str,
  });
  let body = json!(body);
  let res = post(format!("{}/new", URL), body);
  if res.is_ok() {
    println!("Creating new Alan app with id {}...\n", res.unwrap());
    status();
  } else {
    println!("Failed to create a new app.");
  }
}

pub fn upgrade(app_id: &str, agz_file: &str) {
  let app_str = get_app_str(agz_file);
  let body = json!({
    "deployConfig": get_config(),
    "appId": app_id,
    "agzB64": app_str,
  });
  let res = post(format!("{}/upgrade", URL), body);
  if res.is_ok() {
    println!("Upgrading Alan app {}...\n", app_id);
  } else {
    println!("Failed to upgrade Alan app {}", app_id);
  }
}

pub fn status() {
  let body = json!({
    "deployConfig": get_config(),
  });
  let resp = post(format!("{}/status", URL), body).unwrap();
  let mut apps: Vec<App> = from_str(resp.as_str()).unwrap();

  if apps.len() == 0 {
    println!("No Alan apps deployed using the cloud credentials in {}", CONFIG_NAME);
    return;
  }

  let mut ascii_table = AsciiTable::default();
  ascii_table.max_width = 100;

  let mut column = Column::default();
  column.header = "Alan App Id".into();
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

  println!("Status of all apps deployed using the cloud credentials in {}\n", CONFIG_NAME);
  ascii_table.print(data);
}
