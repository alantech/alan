use std::env;
use std::fs::read;

use anycloud::deploy;
use base64;
use serde::Serialize;
use serde_json::{json, Value};
use futures::future::join_all;
use futures::stream::StreamExt;
#[cfg(target_os = "linux")]
use heim_cpu::os::linux::CpuTimeExt;
#[cfg(target_os = "linux")]
use heim_memory::os::linux::MemoryExt;
use heim_process::processes;
use heim_common::units::{information::kilobyte, ratio::ratio, time::second};
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
  procsCpuUsage: Vec<f32>,
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
    Err(err) => format!("{:?}", err),
  }
}

async fn post_v1_scale(cluster_id: &str, agz_b64: &str, deploy_token: &str, factor: &str) -> String {
  // transmit the Dockerfile and app.tar.gz if both are available
  let pwd = env::current_dir();
  match pwd {
    Ok(pwd) => {
      let dockerfile = read(format!("{}/Dockerfile", pwd.display()));
      let app_tar_gz = read(format!("{}/app.tar.gz", pwd.display()));
      let env_file = read(format!("{}/anycloud.env", pwd.display()));
      let mut scale_body = json!({
        "clusterId": cluster_id,
        "agzB64": agz_b64,
        "deployToken": deploy_token,
        "clusterFactor": factor,
      });
      if let (Ok(dockerfile), Ok(app_tar_gz)) = (dockerfile, app_tar_gz) {
        scale_body.as_object_mut().unwrap().insert(format!("DockerfileB64"), json!(base64::encode(dockerfile)));
        scale_body.as_object_mut().unwrap().insert(format!("appTarGzB64"), json!(base64::encode(app_tar_gz)));
      }
      if let Ok(env_file) = env_file {
        scale_body.as_object_mut().unwrap().insert(format!("envB64"), json!(base64::encode(env_file)));
      };
      post_v1("scale", scale_body).await
    },
    Err(err) => format!("{:?}", err),
  }
}

async fn get_procs_cpu_usage() -> Vec<f32> {
  let futures = processes()
    .map(|process| async {
      match process {
        Ok(proc) => {
          let measurement_1 = proc.cpu_usage().await;
          // CpuUsage struct represents instantaneous CPU usage and
          // does not represent any reasonable value by itself
          sleep(Duration::from_secs(1)).await;
          let measurement_2 = proc.cpu_usage().await;
          if measurement_1.is_err() || measurement_2.is_err() {
            return 0.0
          }
          let usage = measurement_2.unwrap() - measurement_1.unwrap();
          usage.get::<ratio>()
        },
        Err(_) => 0.0
      }
    })
    .collect::<Vec<_>>()
    .await;
  join_all(futures).await
}

#[cfg(target_os = "linux")]
async fn get_v1_stats() -> VMStatsV1 {
  let memory = heim_memory::memory().await.expect("Failed to get system memory information");
  let swap = heim_memory::swap().await.expect("Failed to get swap information");
  VMStatsV1 {
    cpuSecs: heim_cpu::times().map(|r| {
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
    procsCpuUsage: get_procs_cpu_usage().await,
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

// zero out linux specific stats
#[cfg(not(target_os = "linux"))]
async fn get_v1_stats() -> VMStatsV1 {
  let memory = heim_memory::memory().await.expect("Failed to get system memory information");
  let swap = heim_memory::swap().await.expect("Failed to get swap information");
  VMStatsV1 {
    cpuSecs: heim_cpu::times().map(|r| {
      let cpu = r.expect("Failed to get CPU times");
      CPUSecsV1 {
        user: cpu.user().get::<second>(),
        system: cpu.system().get::<second>(),
        idle: cpu.idle().get::<second>(),
        irq: 0.0,
        nice: 0.0,
        ioWait: 0.0,
        softIrq: 0.0,
        steal: 0.0,
      }
    }).collect().await,
    procsCpuUsage: get_procs_cpu_usage().await,
    totalMemoryKb: memory.total().get::<kilobyte>(),
    availableMemoryKb: memory.available().get::<kilobyte>(),
    freeMemoryKb: memory.free().get::<kilobyte>(),
    activeMemoryKb: 0,
    usedMemoryKb: 0,
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
