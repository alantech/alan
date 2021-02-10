use tokio::task;
use tokio::time::{Duration, sleep};

use crate::vm::deploy::vm_health;
use crate::vm::run::run;

pub async fn start(agz_file: &str, app_id: &str, deploy_key: &str) {
  let app_id = app_id.to_string();
  let deploy_key = deploy_key.to_string();
  task::spawn(async move {
    // TODO better period determination https://github.com/alantech/deploy/issues/14
    let period = Duration::from_secs(30);
    loop {
      vm_health(&app_id, &deploy_key).await;
      sleep(period).await
    }
  });
  run(agz_file, true).await;
}