use anycloud::deploy;
use serde::Serialize;
use serde_json::{json, Value};
use futures::stream::StreamExt;
use heim::{
  cpu::{self, os::linux::CpuTimeExt},
  memory::{self, os::linux::MemoryExt},
  units::{information::kilobyte, time::second}
};
use tokio::process::Command;
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

async fn get_private_ip() -> String {
  let res = Command::new("hostname")
    .arg("-I")
    .output()
    .await;
  let err = "Failed to execute `hostname`";
  let stdout = res.expect(err).stdout;
  String::from_utf8(stdout)
    .expect(err)
    .trim()
    .to_string()
}

async fn post_v1(endpoint: &str, body: Value) -> String {
  let resp = deploy::post_v1(endpoint, body).await;
  match resp {
    Ok(res) => res,
    Err(err) => err.to_string(),
  }
}

async fn post_v1_scale(cluster_id: &str, agz_b64: &str, deploy_token: &str, alan_version: &str, delta: i32) -> String {
  let scale_body = json!({
    "alanVersion": alan_version,
    "clusterId": cluster_id,
    "agzB64": agz_b64,
    "deployToken": deploy_token,
    "clusterDelta": delta,
  });
  post_v1("scale", scale_body).await
}

async fn get_v1_stats() -> VMStatsV1 {
  let memory = memory::memory().await.expect("Failed to get system memory information");
  let swap = memory::swap().await.expect("Failed to get swap information");
  VMStatsV1 {
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
  }
}

// returns cluster delta
async fn post_v1_stats(deploy_token: &str, cluster_size: usize) -> i32 {
  let vm_stats = get_v1_stats().await;
  let stats_body = json!({
    "deployToken": deploy_token,
    "vmStats": vm_stats,
    "clusterSize": cluster_size,
  });
  let resp = post_v1("stats", stats_body).await;
  resp.parse().expect("Failed to parse cluster delta")
}

pub async fn start(cluster_id: &str, agz_b64: &str, deploy_token: &str) {
  let cluster_id = cluster_id.to_string();
  let deploy_token = deploy_token.to_string();
  let agzb64 = agz_b64.to_string();
  task::spawn(async move {
    let dns = DNS::new("alandeploy.com");
    let vms = dns.get_vms(&cluster_id).await;
    let cluster_size = vms.len();
    let ips = vms.iter().map(|vm| vm.private_ip_addr.to_string()).collect();
    let alan_version = &vms[0].alan_version;
    let lrh = LogRendezvousHash::new(ips);
    let leader_ip = lrh.get_leader_id();
    let self_ip = get_private_ip().await;
    if leader_ip == self_ip {
      // TODO better period determination
      let period = Duration::from_secs(30);
      loop {
        let delta = post_v1_stats(&deploy_token, cluster_size).await;
        println!("VM stats sent for cluster {} of size {}. Cluster delta: {}.", cluster_id, cluster_size, delta);
        if delta != 0 {
          post_v1_scale(&cluster_id, &agzb64, &deploy_token, alan_version, delta).await;
        }
        // TODO maybe ask deploy service to /upgrade if delta = 0 and not on the latest alan version
        sleep(period).await
      }
    }
  });
  run_agz_b64(agz_b64).await;
}