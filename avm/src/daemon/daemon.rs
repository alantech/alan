use anycloud::deploy::post_v1;
use serde::Serialize;
use serde_json::{json, Value};
use sysinfo::{ProcessorExt, SystemExt};
use tokio::process::Command;
use tokio::task;
use tokio::time::{Duration, sleep};

use crate::daemon::dns::DNS;
use crate::daemon::lrh::LogRendezvousHash;
use crate::vm::run::run_agz_b64;

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
struct VMStats {
  schemaVersion: String,
  cpuUsage: Vec<f32>,
  totalMemory: u64,
  usedMemory: u64,
  availableMemory: u64,
  freeMemory: u64,
  totalSwap: u64,
  usedSwap: u64,
  freeSwap: u64,
}

pub async fn get_ip() -> String {
  let res = Command::new("hostname")
    .arg("-I")
    .output()
    .await;
  let err = "Failed to execute `hostname`";
  String::from_utf8(res.expect(err).stdout).expect(err)
}

pub async fn safe_post(endpoint: &str, body: Value) -> String {
  let resp = post_v1(endpoint, body).await;
  match resp {
    Ok(res) => res,
    Err(err) => err.to_string(),
  }
}

pub async fn start(agz_b64: &str, app_id: &str, deploy_token: &str) {
  let app_id = app_id.to_string();
  let deploy_token = deploy_token.to_string();
  let agz_b64 = agz_b64.to_string();
  task::spawn(async move {
    let dns = DNS::new("alandeploy.com");
    let ip_addrs = dns.get_ip_addrs(&app_id).await;
    let cluster_size = ip_addrs.len();
    let lrh = LogRendezvousHash::new(ip_addrs);
    let leader_ip = lrh.get_leader_id();
    let self_ip = get_ip().await;
    if true {
    //if leader_ip == self_ip {
      let mut system = sysinfo::System::new_all();
      // TODO better period determination
      let period = Duration::from_secs(30);
      loop {
        // First we update all information of our system struct.
        system.refresh_all();
    
        let stats = VMStats {
          schemaVersion: "v0".to_string(),
          cpuUsage: system.get_processors().iter().map(|p| p.get_cpu_usage()).collect(),
          totalMemory: system.get_total_memory(),
          usedMemory: system.get_used_memory(),
          availableMemory: system.get_available_memory(),
          freeMemory: system.get_free_memory(),
          totalSwap: system.get_total_swap(),
          usedSwap: system.get_used_swap(),
          freeSwap: system.get_free_swap(),
        };
        
        let stats_body = json!({
          "deployToken": deploy_token,
          "vmStats": stats,
          "clusterSize": cluster_size,
        });
    
        let resp = safe_post("stats", stats_body).await;
        let delta: i32 = resp.parse().unwrap();
        if delta != 0 {
          let update_body = json!({
            "appId": app_id,
            "agzB64": agz_b64,
            "deployToken": deploy_token,
            "clusterDelta": delta,
          });
          safe_post("scale", update_body).await;
        }
        sleep(period).await
      }
    }
    run_agz_b64(&agz_b64).await;
  });
}