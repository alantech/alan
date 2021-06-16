use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::read;
use std::io::Read;

use anycloud::common::{file_exist, get_app_tar_gz_b64, get_dockerfile_b64};
use anycloud::deploy;
use anycloud::{error, CLUSTER_ID};
use base64;
use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use lazy_static::lazy_static;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::sync::watch::{self, Receiver};
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
pub static CONTROL_PORT_CHANNEL: OnceCell<Receiver<ControlPort>> = OnceCell::new();

lazy_static! {
  pub static ref ALAN_TECH_ENV: String =
    std::env::var("ALAN_TECH_ENV").unwrap_or("production".to_string());
}

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
  match ALAN_TECH_ENV.as_str() {
    "local" => Ok("127.0.0.1".to_string()),
    _ => {
      let res = tokio::process::Command::new("hostname")
        .arg("-I")
        .output()
        .await?;
      let stdout = res.stdout;
      let private_ip = String::from_utf8(stdout)?;
      match private_ip.trim().split_whitespace().next() {
        Some(private_ip) => Ok(private_ip.to_string()),
        None => Err("No ip found".into()),
      }
    }
  }
}

#[cfg(not(target_os = "linux"))]
async fn get_private_ip() -> DaemonResult<String> {
  match ALAN_TECH_ENV.as_str() {
    "local" => Ok("127.0.0.1".to_string()),
    _ => Err("`hostname` command does not exist in this OS".into()),
  }
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
            if let Err(err) = run(
              bytecode,
              HttpType::HTTPS(HttpsConfig {
                port: 443,
                priv_key: String::from_utf8(priv_key).unwrap(),
                cert: String::from_utf8(cert).unwrap(),
              }),
            )
            .await
            {
              return Err(format!("Run server has failed. {}", err).into());
            }
          } else {
            return Err("AGZ file appears to be corrupt.".into());
          }
        } else {
          return Err("No self-signed certificate".into());
        }
      }
      Err(err) => {
        return Err(format!("{:?}", err).into());
      }
    }
  } else {
    return Err("AGZ payload not properly base64-encoded.".into());
  }
  Ok(())
}

async fn get_files_b64(is_local_anycloud_app: bool) -> HashMap<String, String> {
  let mut files_b64 = HashMap::new();
  // Check for AnyCloud files
  if is_local_anycloud_app {
    files_b64.insert("Dockerfile".to_string(), get_dockerfile_b64().await);
    files_b64.insert("app.tar.gz".to_string(), get_app_tar_gz_b64(false).await);
  }
  files_b64
}

async fn generate_token() -> String {
  let body = json!({
    "deployConfig": deploy::get_config("", false).await,
  });
  post_v1("localDaemonToken", body).await
}

async fn set_local_daemon_props(is_local_anycloud_app: bool, local_agz_b64: Option<String>) -> () {
  let files_b64 = get_files_b64(is_local_anycloud_app).await;
  let agz_b64 = if let Some(local_agz_b64) = local_agz_b64 {
    local_agz_b64
  } else {
    eprintln!(
      "running the Alan Daemon in a local environment requires the \
               --agz-file argument specifying the .agz file to run"
    );
    std::process::exit(1);
  };
  DAEMON_PROPS
    .set(DaemonProperties {
      clusterId: "daemon-local-cluster".to_string(),
      agzB64: agz_b64,
      deployToken: generate_token().await,
      domain: "alandeploy.com".to_string(),
      filesB64: files_b64,
    })
    .unwrap();
}

fn maybe_create_certs() {
  if !file_exist("key.pem") && !file_exist("certificate.pem") {
    // Self signed certs for local dev
    // openssl req -newkey rsa:2048 -nodes -keyout key.pem -x509 -days 365 -out certificate.pem -subj "/C=US/ST=California/O=Alan Technologies, Inc/CN=*.anycloudapp.com"
    let mut open_ssl = std::process::Command::new("openssl")
      .stdout(std::process::Stdio::null())
      .stderr(std::process::Stdio::null())
      .arg("req")
      .arg("-newkey")
      .arg("rsa:2048")
      .arg("-nodes")
      .arg("-keyout")
      .arg("key.pem")
      .arg("-x509")
      .arg("-days")
      .arg("365")
      .arg("-out")
      .arg("certificate.pem")
      .arg("-subj")
      .arg("/C=US/ST=California/O=Alan Technologies, Inc/CN=*.anycloudapp.com")
      .spawn()
      .expect("Error generating self signed certificate");
    open_ssl.wait().expect("Failed to wait on child");
  }
}

async fn get_daemon_props(
  is_local_anycloud_app: bool,
  local_agz_b64: Option<String>,
) -> Option<&'static DaemonProperties> {
  if ALAN_TECH_ENV.as_str() == "local" {
    set_local_daemon_props(is_local_anycloud_app, local_agz_b64).await;
    return DAEMON_PROPS.get();
  }
  let duration = Duration::from_secs(10);
  let mut counter: u8 = 0;
  // Check every 10s over 10 min if props are ready
  while counter < 60 {
    if let Some(props) = DAEMON_PROPS.get() {
      return Some(props);
    }
    counter += 1;
    sleep(duration).await;
  }
  None
}

pub async fn start(is_local_anycloud_app: bool, local_agz_b64: Option<String>) {
  maybe_create_certs();
  let mut control_port = ControlPort::start().await;
  let (ctrl_tx, ctrl_rx) = watch::channel(control_port.clone());
  CONTROL_PORT_CHANNEL.set(ctrl_rx).unwrap();
  if let Some(daemon_props) = get_daemon_props(is_local_anycloud_app, local_agz_b64).await {
    let cluster_id = &daemon_props.clusterId;
    CLUSTER_ID.set(String::from(cluster_id)).unwrap();
    let domain = &daemon_props.domain;
    let deploy_token = &daemon_props.deployToken;
    let agz_b64 = &daemon_props.agzB64;
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
              error!(NoDnsVms, "{}", err).await;
              None
            }
          };
          // TODO: Figure out how to avoid flushing the LogRendezvousHash table every iteration, but
          // avoid bugs with misidentifying cluster changes as not-changed
          if let Some(vms) = vms {
            cluster_size = vms.len();
            control_port.update_vms(self_ip, vms).await;
            ctrl_tx.send(control_port.clone()).unwrap();
          }
          if control_port.is_leader() {
            // TODO: Should we keep these leader announcements in the stdout logging?
            println!("I am leader!");
            // Do not collect stats until cluster is up. Otherwise, will have scaling issues.
            if control_port.is_cluster_up() {
              match get_v1_stats().await {
                Ok(s) => stats.push(s),
                Err(err) => error!(NoStats, "{}", err).await,
              };
            } else {
              println!("Cluster is not ready. Do not collect stats");
            }
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
              error!(PostFailed, "{}", err).await;
            }
            println!(
              "VM stats sent for cluster {} of size {}. Cluster factor: {}.",
              cluster_id, cluster_size, factor
            );
            if factor != "1" {
              post_v1_scale(&cluster_id, &agz_b64, &deploy_token, &factor).await;
            }
          }
          control_port.check_cluster_health().await;
          sleep(period).await;
        }
      } else if let Err(dns_err) = &dns {
        error!(NoDns, "DNS error: {}", dns_err).await;
        std::process::exit(1);
      } else if let Err(self_ip_err) = &self_ip {
        error!(NoPrivateIp, "Private ip error: {}", self_ip_err).await;
        std::process::exit(1);
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
