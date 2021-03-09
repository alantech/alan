use std::env::var;
use std::fs::read;

use anycloud::deploy;
use base64;
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

#[allow(non_snake_case)]
#[derive(Debug, Serialize)]
struct CPUSecsV1 {
  user: f64,
  system: f64,
  idle: f64,
  irq: f64,
  nice: f64,
  ioWait: f64,
  softIrq: f64,
  steal: f64,
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
    .split_whitespace()
    .next()
    .unwrap()
    .to_string()
}

async fn post_v1(endpoint: &str, body: Value) -> String {
  let resp = deploy::post_v1(endpoint, body).await;
  match resp {
    Ok(res) => res,
    Err(err) => err.to_string(),
  }
}

async fn post_v1_scale(cluster_id: &str, agz_b64: &str, deploy_token: &str, factor: &str) -> String {
  // transmit the Dockerfile and app.tar.gz if both are available
  let pwd = var("PWD").unwrap();
  let dockerfile = read(format!("{}/Dockerfile", pwd));
  let app_tar_gz = read(format!("{}/app.tar.gz", pwd));
  let scale_body = if dockerfile.is_ok() && app_tar_gz.is_ok() {
    json!({
      "clusterId": cluster_id,
      "agzB64": agz_b64,
      "deployToken": deploy_token,
      "clusterFactor": factor,
      "DockerfileB64": base64::encode(dockerfile.unwrap()),
      "appTarGzB64": base64::encode(app_tar_gz.unwrap()),
    })
  } else {
    json!({
      "clusterId": cluster_id,
      "agzB64": agz_b64,
      "deployToken": deploy_token,
      "clusterFactor": factor,
    })
  };
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
        irq: cpu.irq().get::<second>(),
        nice: cpu.nice().get::<second>(),
        ioWait: cpu.io_wait().get::<second>(),
        softIrq: cpu.soft_irq().get::<second>(),
        steal: cpu.steal().get::<second>(),
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
async fn post_v1_stats(cluster_id: &str, deploy_token: &str) -> String {
  let vm_stats = get_v1_stats().await;
  let stats_body = json!({
    "deployToken": deploy_token,
    "vmStats": vm_stats,
    "clusterId": cluster_id,
  });
  post_v1("stats", stats_body).await
}

pub async fn start(
  cluster_id: &str,
  agz_b64: &str,
  deploy_token: &str,
  domain: &str,
  priv_key_b64: Option<&str>,
  cert_b64: Option<&str>,
) {
  let cluster_id = cluster_id.to_string();
  let deploy_token = deploy_token.to_string();
  let agzb64 = agz_b64.to_string();
  let domain = domain.to_string();
  task::spawn(async move {
    // TODO better period determination
    let period = Duration::from_secs(180);
    let dns = DNS::new(&domain);
    let self_ip = get_private_ip().await;
    let mut cluster_size = 0;
    let mut leader_ip = "".to_string();
    loop {
      let vms = dns.get_vms(&cluster_id).await;
      // triggered the first time since cluster_size == 0
      // and every time the cluster changes size
      if vms.len() != cluster_size {
        cluster_size = vms.len();
        let ips = vms.iter().map(|vm| vm.private_ip_addr.to_string()).collect();
        let lrh = LogRendezvousHash::new(ips);
        leader_ip = lrh.get_leader_id().to_string();
      }
      if leader_ip == self_ip {
        let factor = post_v1_stats(&cluster_id, &deploy_token).await;
        println!("VM stats sent for cluster {} of size {}. Cluster factor: {}.", cluster_id, vms.len(), factor);
        if factor != "1" {
          post_v1_scale(&cluster_id, &agzb64, &deploy_token, &factor).await;
        }
      }
      sleep(period).await
    }
  });
  run_agz_b64(agz_b64, priv_key_b64, cert_b64).await;
}
