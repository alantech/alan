use std::convert::Infallible;
use std::env;
use std::fs::read;
use std::io::Read;
use std::net::TcpStream;
use std::path::Path;

use anycloud::deploy;
use base64;
use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use futures::future::join_all;
use futures::stream::StreamExt;
use heim_common::units::{information::kilobyte, ratio::ratio, time::second};
#[cfg(target_os = "linux")]
use heim_cpu::os::linux::CpuTimeExt;
#[cfg(target_os = "linux")]
use heim_memory::os::linux::MemoryExt;
use heim_process::processes;
use hyper::{Body, Request, Response};
use once_cell::sync::OnceCell;
use serde::Serialize;
use serde_json::{json, Value};
use tokio::process::Command;
use tokio::task;
use tokio::time::{sleep, Duration};

use crate::daemon::dns::DNS;
use crate::daemon::lrh::LogRendezvousHash;
use crate::make_server;
use crate::vm::http::{HttpConfig, HttpType, HttpsConfig};
use crate::vm::run::run;

pub static SECRET_STRING: OnceCell<Option<String>> = OnceCell::new();

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
  let res = Command::new("hostname").arg("-I").output().await;
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

async fn post_v1_scale(
  cluster_id: &str,
  agz_b64: &str,
  deploy_token: &str,
  factor: &str,
) -> String {
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
        scale_body
          .as_object_mut()
          .unwrap()
          .insert(format!("DockerfileB64"), json!(base64::encode(dockerfile)));
        scale_body
          .as_object_mut()
          .unwrap()
          .insert(format!("appTarGzB64"), json!(base64::encode(app_tar_gz)));
      }
      if let Ok(env_file) = env_file {
        scale_body
          .as_object_mut()
          .unwrap()
          .insert(format!("envB64"), json!(base64::encode(env_file)));
      };
      post_v1("scale", scale_body).await
    }
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
            return 0.0;
          }
          let usage = measurement_2.unwrap() - measurement_1.unwrap();
          usage.get::<ratio>()
        }
        Err(_) => 0.0,
      }
    })
    .collect::<Vec<_>>()
    .await;
  join_all(futures).await
}

#[cfg(target_os = "linux")]
async fn get_v1_stats() -> VMStatsV1 {
  let memory = heim_memory::memory()
    .await
    .expect("Failed to get system memory information");
  let swap = heim_memory::swap()
    .await
    .expect("Failed to get swap information");
  VMStatsV1 {
    cpuSecs: heim_cpu::times()
      .map(|r| {
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
      })
      .collect()
      .await,
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
  let memory = heim_memory::memory()
    .await
    .expect("Failed to get system memory information");
  let swap = heim_memory::swap()
    .await
    .expect("Failed to get swap information");
  VMStatsV1 {
    cpuSecs: heim_cpu::times()
      .map(|r| {
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
      })
      .collect()
      .await,
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
  let mut stats_body = json!({
    "deployToken": deploy_token,
    "vmStats": vm_stats,
    "clusterId": cluster_id,
  });
  let secret_string = SECRET_STRING.get().unwrap();
  if let Some(secret_string) = secret_string.as_ref() {
    stats_body
      .as_object_mut()
      .unwrap()
      .insert("secretString".to_string(), json!(secret_string));
  }
  post_v1("stats", stats_body).await
}

async fn control_port(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let secret_string = SECRET_STRING.get().unwrap();
  if secret_string.is_some() && !req.headers().contains_key(secret_string.as_ref().unwrap()) {
    // If this control port is guarded by a secret string, make sure there's a header with that
    // secret as the key (we don't care about the value) and abort otherwise
    Ok(Response::builder().status(500).body("fail".into()).unwrap())
  } else if TcpStream::connect("127.0.0.1:443").is_err() {
    // If the Alan HTTPS server has not yet started, mark as a failure
    Ok(Response::builder().status(500).body("fail".into()).unwrap())
  } else if Path::new("./Dockerfile").exists()
    && Path::new("./app.tar.gz").exists()
    && TcpStream::connect("127.0.0.1:8088").is_err()
  {
    // If this is an Anycloud deployment and the child process hasn't started, mark as a failure
    // TODO: Any way to generalize this so we don't have special logic for Anycloud?
    Ok(Response::builder().status(500).body("fail".into()).unwrap())
  } else {
    // Everything passed, send an ok
    Ok(Response::builder().status(200).body("ok".into()).unwrap())
  }
}

async fn run_agz_b64(agz_b64: &str, priv_key_b64: Option<&str>, cert_b64: Option<&str>) {
  let bytes = base64::decode(agz_b64).unwrap();
  let agz = GzDecoder::new(bytes.as_slice());
  let count = agz.bytes().count();
  let mut bytecode = vec![0; count / 8];
  let mut gz = GzDecoder::new(bytes.as_slice());
  gz.read_i64_into::<LittleEndian>(&mut bytecode).unwrap();
  if let (Some(priv_key_b64), Some(cert_b64)) = (priv_key_b64, cert_b64) {
    // Spin up a control port if we can start a secure connection
    make_server!(
      HttpType::HTTPS(HttpsConfig {
        port: 4142, // 4 = A, 1 = L, 2 = N (sideways) => ALAN
        priv_key_b64: priv_key_b64.to_string(),
        cert_b64: cert_b64.to_string(),
      }),
      control_port
    );
    run(
      bytecode,
      HttpType::HTTPS(HttpsConfig {
        port: 443,
        priv_key_b64: priv_key_b64.to_string(),
        cert_b64: cert_b64.to_string(),
      }),
    )
    .await;
  } else {
    run(bytecode, HttpType::HTTP(HttpConfig { port: 80 })).await;
  }
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
        let ips = vms
          .iter()
          .map(|vm| vm.private_ip_addr.to_string())
          .collect();
        let lrh = LogRendezvousHash::new(ips);
        leader_ip = lrh.get_leader_id().to_string();
      }
      if leader_ip == self_ip {
        let factor = post_v1_stats(&cluster_id, &deploy_token).await;
        println!(
          "VM stats sent for cluster {} of size {}. Cluster factor: {}.",
          cluster_id,
          vms.len(),
          factor
        );
        if factor != "1" {
          post_v1_scale(&cluster_id, &agzb64, &deploy_token, &factor).await;
        }
      }
      sleep(period).await
    }
  });
  run_agz_b64(agz_b64, priv_key_b64, cert_b64).await;
}
