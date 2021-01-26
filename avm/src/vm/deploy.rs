use hyper::{client::Client, Body, Request};
use serde::Deserialize;
use serde_json::{from_reader, from_str, json, Value};

use std::error::Error;
use std::fmt::Display;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use ascii_table::{AsciiTable, Column, Align};
use tokio::runtime::Runtime;

#[derive(Deserialize, Debug)]
struct Config {
  access_key_id: String,
  secret_access_key: String,
}

#[derive(Deserialize, Debug)]
struct App {
  id: String,
  vms: Vec<VM>,
}

#[derive(Deserialize, Debug)]
struct VM {
  id: String,
  url: String,
  status: String,
}

const URL: &str = "http://localhost:8000";

fn get_config() -> Result<Config, Box<dyn Error>> {
  let file_name = "alan-deploy.json";
  let path = Path::new(file_name);
  if !path.exists() {
    println!(
      "{} does not exist. Please define one.",
      file_name
    );
    std::process::exit(1);
  }
  let file = File::open(path)?;
  let reader = BufReader::new(file);
  let c = from_reader(reader)?;
  Ok(c)
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

pub fn kill(app_id: &str) {
  let config = get_config().unwrap();
  let body = json!({
    "aws": {
      "accessKeyId": config.access_key_id,
      "secretAccessKey": config.secret_access_key,
    },
    "appId": app_id,
  });
  let res = post(format!("{}/kill", URL), body);
  if res.is_ok() {
    println!("Killing {} ...", app_id);
  } else {
    println!("Killing {} failed", app_id);
  }
}

pub fn new() {
  // let dest_file = "temp.agz";
  // let status_code = compile(&ln_file, &dest_file, true);
  // if status_code == 1 {
  //   std::process::exit(status_code);
  // }
  let config = get_config().unwrap();
  let body = json!({
    "aws": {
      "accessKeyId": config.access_key_id,
      "secretAccessKey": config.secret_access_key,
    },
  });
  let res = post(format!("{}/new", URL), body);
  if res.is_ok() {
    println!("Creating new app with id {}...", res.unwrap());
  } else {
    println!("Failed to create a new app");
  }
}

pub fn upgrade(app_id: &str) {
  // let dest_file = "temp.agz";
  // let status_code = compile(&ln_file, &dest_file, true);
  // if status_code == 1 {
  //   std::process::exit(status_code);
  // }
  let config = get_config().unwrap();
  let body = json!({
    "aws": {
      "accessKeyId": config.access_key_id,
      "secretAccessKey": config.secret_access_key,
    },
    "appId": app_id,
  });
  let res = post(format!("{}/new", URL), body);
  if res.is_ok() {
    println!("Upgrading code for app with id {}...", app_id);
  } else {
    println!("Failed to upgrade {}", app_id);
  }
}

pub fn status() {
  let config = get_config().unwrap();
  let body = json!({
    "aws": {
      "accessKeyId": config.access_key_id,
      "secretAccessKey": config.secret_access_key,
    }
  });
  let resp = post(format!("{}/status", URL), body).unwrap();
  let mut apps: Vec<App> = from_str(resp.as_str()).unwrap();

  let mut ascii_table = AsciiTable::default();

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
    data.push(vec![&app.id, &app.vms[0].status, &app.vms[0].url]);
  }

  ascii_table.print(data);
}
