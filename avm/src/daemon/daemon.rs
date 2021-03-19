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
use hyper::{Body, Request, Response};
use once_cell::sync::OnceCell;
use serde_json::{json, Value};
use tokio::process::Command;
use tokio::task;
use tokio::time::{sleep, Duration};

use crate::daemon::dns::DNS;
use crate::daemon::lrh::LogRendezvousHash;
use crate::daemon::stats::get_v1_stats;
use crate::make_server;
use crate::vm::http::{HttpConfig, HttpType, HttpsConfig};
use crate::vm::run::run;

pub static CLUSTER_SECRET: OnceCell<Option<String>> = OnceCell::new();

#[cfg(target_os = "linux")]
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

#[cfg(not(target_os = "linux"))]
async fn get_private_ip() -> String {
  panic!("`hostname` command does not exist in this OS");
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

// returns cluster delta
async fn post_v1_stats(cluster_id: &str, deploy_token: &str) -> String {
  let vm_stats = get_v1_stats().await;
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
  }
  post_v1("stats", stats_body).await
}

async fn control_port(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let cluster_secret = CLUSTER_SECRET.get().unwrap();
  if cluster_secret.is_some() && !req.headers().contains_key(cluster_secret.as_ref().unwrap()) {
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
    // TODO even better period determination
    let period = Duration::from_secs(5 * 60);
    let dns = DNS::new(&domain);
    let self_ip = get_private_ip().await;
    let mut cluster_size = 0;
    let mut leader_ip = "".to_string();
    loop {
      sleep(period).await;
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
    }
  });
  run_agz_b64(agz_b64, priv_key_b64, cert_b64).await;
}
