use once_cell::sync::OnceCell;
use tokio::task;
use tokio::time::{Duration, sleep};

use crate::vm::deploy::vm_health;
use crate::vm::run::run;

pub static APP_ID: OnceCell<String> = OnceCell::new();
pub static DEPLOY_KEY: OnceCell<String> = OnceCell::new();

pub async fn start(agz_file: &str, app_id: &str, deploy_key: &str) {
  let set_app_id = APP_ID.set(app_id.to_string());
  let set_deploy_key = DEPLOY_KEY.set(deploy_key.to_string());
  if set_app_id.is_err() || set_deploy_key.is_err() {
    eprintln!("Failed to globally set app id or deploy key");
    std::process::exit(1);
  }
  let _app_id_global = APP_ID.get().unwrap();
  let _deploy_key_global = DEPLOY_KEY.get().unwrap();
  task::spawn(async move {
    let ten_min_period = Duration::from_secs(60*10);
    loop {
      // TODO use rendezvous hashing to determine if self is leader
      // send vm health stats
      sleep(ten_min_period).await
    }
  });
  run(agz_file, true).await;
}