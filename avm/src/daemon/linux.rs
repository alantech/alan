use anycloud::deploy::post_v1;
use serde::Serialize;
use serde_json::{json, Value};
use futures::stream::StreamExt;
use heim::{
  host,
  cpu::{self, os::linux::CpuTimeExt},
  memory::{self, os::linux::MemoryExt},
  units::{information::kilobyte, time::second}
};
use tokio::task;
use tokio::time::{Duration, sleep};

use crate::daemon::dns::DNS;
use crate::daemon::lrh::LogRendezvousHash;
use crate::vm::run::run_agz_b64;

#[derive(Debug, Serialize)]
struct CPUSecsV1 {
  user: f64,
  system: f64,
  idle: f64,
  interrupt: f64,
  nice: f64,
  iowait: f64,
}

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
struct VMStatsV1 {
  cpuSecs: Vec<CPUSecsV1>,
  totalMemoryKb: u64,
  availableMemoryKb: u64,
  freeMemoryKb: u64,
  usedMemoryKb: u64,
  activeMemoryKb: u64,
  totalSwapKb: u64,
  usedSwapKb: u64,
  freeSwapKb: u64,
}

pub async fn post(endpoint: &str, body: Value) -> String {
  let resp = post_v1(endpoint, body).await;
  match resp {
    Ok(res) => res,
    Err(err) => err.to_string(),
  }
}

pub async fn start(app_id: &str, agz_b64: &str, deploy_token: &str) {
  let app_id = app_id.to_string();
  let deploy_token = deploy_token.to_string();
  let agzb64 = agz_b64.to_string();
  task::spawn(async move {
    let dns = DNS::new("alandeploy.com");
    let ip_addrs = dns.get_ip_addrs(&app_id).await;
    let cluster_size = ip_addrs.len();
    let lrh = LogRendezvousHash::new(ip_addrs);
    let leader_ip = lrh.get_leader_id();
    let platform = host::platform().await.expect("Failed to get platform information");
    let self_ip = platform.hostname().replace("ip-", "");
    if leader_ip == self_ip {
      // TODO better period determination
      let period = Duration::from_secs(30);
      loop {
        let memory = memory::memory().await.expect("Failed to get system memory information");
        let swap = memory::swap().await.expect("Failed to get swap information");
        let stats = VMStatsV1 {
          cpuSecs: cpu::times().map(|r| {
            let cpu = r.expect("Failed to get CPU times");
            CPUSecsV1 {
              user: cpu.user().get::<second>(),
              system: cpu.system().get::<second>(),
              idle: cpu.idle().get::<second>(),
              interrupt: cpu.irq().get::<second>(),
              nice: cpu.nice().get::<second>(),
              iowait: cpu.io_wait().get::<second>(),
            }
          }).collect().await,
          totalMemoryKb: memory.total().get::<kilobyte>(),
          availableMemoryKb: memory.available().get::<kilobyte>(),
          freeMemoryKb: memory.free().get::<kilobyte>(),
          activeMemoryKb: memory.active().get::<kilobyte>(),
          usedMemoryKb: memory.used().get::<kilobyte>(),
          totalSwapKb: swap.total().get::<kilobyte>(),
          usedSwapKb: swap.used().get::<kilobyte>(),
          freeSwapKb: swap.free().get::<kilobyte>(),
        };
        
        let stats_body = json!({
          "deployToken": deploy_token,
          "vmStats": stats,
          "clusterSize": cluster_size,
        });
    
        let resp = post("stats", stats_body).await;
        let delta: i32 = resp.parse().expect("Failed to parse cluster delta");
        if delta != 0 {
          let scale_body = json!({
            "appId": app_id,
            "agzB64": agzb64,
            "deployToken": deploy_token,
            "clusterDelta": delta,
          });
          post("scale", scale_body).await;
        }
        sleep(period).await
      }
    }
  });
  run_agz_b64(agz_b64).await;
}