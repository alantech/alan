use tokio::process::Command;
use tokio::task;
use tokio::time::{Duration, sleep};

use crate::daemon::dns::DNS;
use crate::daemon::lrh::LogRendezvousHash;
use crate::vm::run::run;

pub async fn get_ip() -> String {
  let res = Command::new("hostname")
    .arg("-I")
    .output()
    .await;
  let err = "Failed to execute `hostname`";
  String::from_utf8(res.expect(err).stdout).expect(err)
}

pub async fn start(agz_file: &str, app_id: &str, deploy_token: &str) {
  let app_id = app_id.to_string();
  let _deploy_token = deploy_token.to_string();
  task::spawn(async move {
    let dns = DNS::new("alandeploy.com");
    let ip_addrs = dns.get_ip_addrs(&app_id).await;
    let lrh = LogRendezvousHash::new(ip_addrs);
    let leader_ip = lrh.get_leader_id();
    let self_ip = get_ip().await;

    if leader_ip == self_ip {
      // TODO better period determination
      let period = Duration::from_secs(30);
      loop {
        // TODO send health stats
        // cpu, ram, file descriptors, network utilization, and disk utilization in decreasing importance
        // For everything, we want both the maximums and the current value
        // vm_health(&app_id, &deploy_token).await;
        sleep(period).await
      }
    }
  });
  run(agz_file, true).await;
}