use std::env;
use std::error::Error;
use std::fs::read;
use std::io::Read;

use anycloud::deploy;
use anycloud::logger::ErrorType;
use anycloud::{error, CLUSTER_ID};
use base64;
use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use once_cell::sync::OnceCell;
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
      error!(ErrorType::PostFailed, "{:?}", err).await;
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
      error!(ErrorType::ScaleFailed, "{:?}", err).await;
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
    error!(ErrorType::NoClusterSecret, "No cluster secret found.").await;
  }
  Ok(post_v1("stats", stats_body).await)
}

async fn run_agz_b64(agz_b64: &str, priv_key_b64: &str, cert_b64: &str) -> DaemonResult<()> {
  let bytes = base64::decode(agz_b64);
  if let Ok(bytes) = bytes {
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
          priv_key_b64: priv_key_b64.to_string(),
          cert_b64: cert_b64.to_string(),
        }),
      )
      .await;
    } else {
      return Err("AGZ file appears to be corrupt.".into());
    }
  } else {
    return Err("AGZ payload not properly base64-encoded.".into());
  }
  Ok(())
}

pub async fn start(
  cluster_id: &str,
  agz_b64: &str,
  deploy_token: &str,
  domain: &str,
  priv_key_b64: &str,
  cert_b64: &str,
) {
  let cluster_id = cluster_id.to_string();
  CLUSTER_ID.set(String::from(&cluster_id)).unwrap();
  let deploy_token = deploy_token.to_string();
  let agzb64 = agz_b64.to_string();
  let domain = domain.to_string();
  let mut control_port = ControlPort::start(priv_key_b64, cert_b64).await;
  task::spawn(async move {
    let period = Duration::from_secs(60);
    let mut stats = Vec::new();
    let mut cluster_size = 0;
    let self_ip = get_private_ip().await;
    let dns = DNS::new(&domain);
    if let (Ok(dns), Ok(self_ip)) = (&dns, &self_ip) {
      loop {
        let vms = match dns.get_vms(&cluster_id).await {
          Ok(vms) => Some(vms),
          Err(err) => {
            error!(ErrorType::NoDnsVms, "{}", err).await;
            None
          }
        };
        println!("vms from dns {:?}", &vms);
        // TODO: Figure out how to avoid flushing the LogRendezvousHash table every iteration, but
        // avoid bugs with misidentifying cluster changes as not-changed
        if let Some(vms) = vms {
          cluster_size = vms.len();
          println!("update_vms is being called!");
          control_port.update_vms(self_ip, vms).await;
        } else {
          println!("update_vms is not being called!");
        }
        if control_port.is_leader() {
          println!("I am leader!");
          match get_v1_stats().await {
            Ok(s) => stats.push(s),
            Err(err) => error!(ErrorType::NoStats, "{}", err).await,
          };
        } else {
          // Debug print for now
          println!("I am NOT the leader! :(");
          println!(
            "Me: {} Leader: {}",
            self_ip,
            control_port
              .get_leader()
              .map(|vm| vm.private_ip_addr.clone())
              .unwrap_or("<None>".to_string())
          );
        }
        if stats.len() >= 4 {
          let mut factor = String::from("1");
          let stats_factor = post_v1_stats(stats.to_owned(), &cluster_id, &deploy_token).await;
          stats = Vec::new();
          if let Ok(stats_factor) = stats_factor {
            factor = stats_factor;
          } else if let Err(err) = stats_factor {
            error!(ErrorType::PostFailed, "{}", err).await;
          }
          println!(
            "VM stats sent for cluster {} of size {}. Cluster factor: {}.",
            cluster_id, cluster_size, factor
          );
          if factor != "1" {
            post_v1_scale(&cluster_id, &agzb64, &deploy_token, &factor).await;
          }
        }
        control_port.check_cluster_health().await;
        sleep(period).await;
      }
    } else if let Err(dns_err) = &dns {
      error!(ErrorType::NoDns, "DNS error: {}", dns_err).await;
      panic!("DNS error: {}", dns_err);
    } else if let Err(self_ip_err) = &self_ip {
      error!(ErrorType::NoPrivateIp, "Private ip error: {}", self_ip_err).await;
      panic!("Private ip error: {}", self_ip_err);
    }
  });
  if let Err(err) = run_agz_b64(agz_b64, priv_key_b64, cert_b64).await {
    error!(ErrorType::RunAgzFailed, "{:?}", err).await;
    panic!("{:?}", err);
  }
}
