use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::read;
use std::io::Read;

use anycloud::deploy;
use anycloud::{error, CLUSTER_ID};
use base64;
use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::process::Command;
use tokio::task;
use tokio::time::{sleep, Duration};

use crate::daemon::ctrl::ControlPort;
use crate::daemon::dns::DNS;
use crate::daemon::stats::{get_v1_stats, VMStatsV1};
use crate::vm::http::{HttpType, HttpsConfig};
use crate::vm::run::run;

pub type DaemonResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

pub static CLUSTER_SECRET: OnceCell<Option<String>> = OnceCell::new();
pub static DAEMON_PROPS: OnceCell<DaemonProperties> = OnceCell::new();

#[allow(non_snake_case)]
#[derive(Deserialize, Debug, Serialize)]
pub struct DaemonProperties {
  pub clusterId: String,
  pub agzB64: String,
  pub deployToken: String,
  pub domain: String,
  pub filesB64: HashMap<String, String>,
}

#[cfg(target_os = "linux")]
async fn get_private_ip() -> DaemonResult<String> {
  let res = Command::new("hostname").arg("-I").output().await?;
  let stdout = res.stdout;
  let private_ip = String::from_utf8(stdout)?;
  match private_ip.trim().split_whitespace().next() {
    Some(private_ip) => Ok(private_ip.to_string()),
    None => Err("No ip found".into()),
  }
}

#[cfg(not(target_os = "linux"))]
async fn get_private_ip() -> DaemonResult<String> {
  Err("`hostname` command does not exist in this OS".into())
}

async fn post_v1(endpoint: &str, body: Value) -> String {
  let resp = deploy::post_v1(endpoint, body).await;
  match resp {
    Ok(res) => res,
    Err(err) => {
      let err = format!("{:?}", err);
      error!(PostFailed, "{:?}", err).await;
      err
    }
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
    Err(err) => {
      let err = format!("{:?}", err);
      error!(ScaleFailed, "{:?}", err).await;
      err
    }
  }
}

// returns cluster delta
async fn post_v1_stats(
  vm_stats: Vec<VMStatsV1>,
  cluster_id: &str,
  deploy_token: &str,
) -> DaemonResult<String> {
  let mut stats_body = json!({
    "deployToken": deploy_token,
    "vmStats": vm_stats,
    "clusterId": cluster_id,
  });
  let cluster_secret = CLUSTER_SECRET.get().unwrap();
  if let Some(cluster_secret) = cluster_secret.as_ref() {
    stats_body
      .as_object_mut()
      .unwrap()
      .insert("clusterSecret".to_string(), json!(cluster_secret));
  } else {
    error!(NoClusterSecret, "No cluster secret found.").await;
  }
  Ok(post_v1("stats", stats_body).await)
}

async fn run_agz_b64(agz_b64: &str) -> DaemonResult<()> {
  let bytes = base64::decode(agz_b64);
  if let Ok(bytes) = bytes {
    let pwd = env::current_dir();
    match pwd {
      Ok(pwd) => {
        let priv_key = read(format!("{}/key.pem", pwd.display()));
        let cert = read(format!("{}/certificate.pem", pwd.display()));
        if let (Ok(priv_key), Ok(cert)) = (priv_key, cert) {
          let agz = GzDecoder::new(bytes.as_slice());
          let count = agz.bytes().count();
          let mut bytecode = vec![0; count / 8];
          let mut gz = GzDecoder::new(bytes.as_slice());

          let gz_read_i64 = gz.read_i64_into::<LittleEndian>(&mut bytecode);
          if gz_read_i64.is_ok() {
            // TODO: return a Result in order to catch the error and log it
            run(
              bytecode,
              HttpType::HTTPS(HttpsConfig {
                port: 443,
                priv_key: String::from_utf8(priv_key).unwrap(),
                cert: String::from_utf8(cert).unwrap(),
              }),
            )
            .await;
          } else {
            return Err("AGZ file appears to be corrupt.".into());
          }
        }
      }
      Err(err) => {
        let err = format!("{:?}", err);
        error!(RunAgzFailed, "{:?}", err).await;
        return Err(err.into());
      }
    }
  } else {
    return Err("AGZ payload not properly base64-encoded.".into());
  }
  Ok(())
}

pub async fn start() {
  let mut control_port = ControlPort::start().await;
  let self_ip = get_private_ip().await;
  match get_private_ip().await {
    Ok(self_ip) => {
      let mut daemon_props: Option<&DaemonProperties> = None;
      let duration = Duration::from_secs(10);
      let mut counter: u8 = 0;
      // Check every 10s over 5 min if props are ready
      while counter < 30 && daemon_props.is_none() {
        if let Some(props) = DAEMON_PROPS.get() {
          daemon_props = Some(props);
        }
        counter += 1;
        sleep(duration).await;
      }
      if let Some(daemon_props) = daemon_props {
        let cluster_id = &daemon_props.clusterId;
        CLUSTER_ID.set(String::from(cluster_id)).unwrap();
        let domain = &daemon_props.domain;
        let deploy_token = &daemon_props.deployToken;
        let agz_b64 = &daemon_props.agzB64;
        task::spawn(async move {
          let period = Duration::from_secs(60);
          let mut stats = Vec::new();
          let mut cluster_size = 0;
          let mut leader_ip = String::new();
          let mut dns = DNS::new(&domain);
          let mut should_update_dns = false;
          loop {
            if should_update_dns {
              dns = DNS::new(&domain);
            }
            if let Ok(dns) = &dns {
              let vms = match dns.get_vms(&cluster_id).await {
                Ok(vms) => {
                  should_update_dns = false;
                  vms
                }
                Err(err) => {
                  should_update_dns = true;
                  error!(NoDnsVms, "{}", err).await;
                  Vec::new()
                }
              };
              // triggered the first time since cluster_size == 0
              // and every time the cluster changes size
              if vms.len() != cluster_size {
                cluster_size = vms.len();
                let ips = vms
                  .iter()
                  .map(|vm| vm.private_ip_addr.to_string())
                  .collect();
                control_port.update_ips(ips);
                leader_ip = control_port.get_leader().to_string();
              }
              if leader_ip == self_ip.to_string() {
                match get_v1_stats().await {
                  Ok(s) => stats.push(s),
                  Err(err) => error!(NoStats, "{}", err).await,
                };
              }
              if stats.len() >= 4 {
                let mut factor = String::from("1");
                let stats_factor =
                  post_v1_stats(stats.to_owned(), &cluster_id, &deploy_token).await;
                stats = Vec::new();
                if let Ok(stats_factor) = stats_factor {
                  factor = stats_factor;
                } else if let Err(err) = stats_factor {
                  error!(PostFailed, "{}", err).await;
                }
                println!(
                  "VM stats sent for cluster {} of size {}. Cluster factor: {}.",
                  cluster_id,
                  vms.len(),
                  factor
                );
                if factor != "1" {
                  post_v1_scale(&cluster_id, &agz_b64, &deploy_token, &factor).await;
                }
              }
              control_port.check_cluster_health().await;
              sleep(period).await;
            } else if let Err(dns_err) = &dns {
              error!(NoDns, "DNS error: {}", dns_err).await;
              std::process::exit(1);
            }
          }
        });
        if let Err(err) = run_agz_b64(&agz_b64).await {
          error!(RunAgzFailed, "{:?}", err).await;
          std::process::exit(1);
        }
      } else {
        let msg = "No daemon properties defined";
        error!(NoDaemonProps, "{}", msg).await;
        std::process::exit(1);
      }
    }
    Err(self_ip_err) => {
      error!(NoPrivateIp, "Private ip error: {}", self_ip_err).await;
      std::process::exit(1);
    }
  }
}
