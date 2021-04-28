use std::cmp;
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::fs::{read, write};
use std::hash::Hasher;
use std::io;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anycloud::error;
use futures::future::join_all;
use hyper::{
  body,
  client::{Client, HttpConnector},
  Body, Request, Response,
};
// TODO: Restore rustls once it can connect directly by IP address
//use hyper_rustls::HttpsConnector;
use hyper_openssl::HttpsConnector;
use once_cell::sync::OnceCell;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
//use rustls::ClientConfig;
use twox_hash::XxHash64;

use crate::daemon::daemon::{DaemonProperties, DaemonResult, CLUSTER_SECRET, DAEMON_PROPS};
use crate::daemon::dns::VMMetadata;
use crate::make_server;
use crate::vm::http::{HttpType, HttpsConfig};
use crate::vm::opcode::REGION_VMS;

pub static NAIVE_CLIENT: OnceCell<Client<HttpsConnector<HttpConnector>>> = OnceCell::new();

#[derive(Debug)]
pub struct HashedId {
  pub id: String,
  pub hash: u64,
  pub is_up: bool,
}

impl HashedId {
  pub fn new(id: String) -> HashedId {
    let mut hasher = XxHash64::with_seed(0xa1a2);
    hasher.write(id.as_bytes());
    HashedId {
      id,
      hash: hasher.finish(),
      is_up: true, // Assume it's up and let the cron job mutate it over time
    }
  }
}

// An algorithm based on the classic Rendezvous Hash but with changes to make the performance
// closer to modern Consistent Hash algorithms while retaining Rendezvous Hash's coordination-free
// routing.
#[derive(Debug)]
pub struct LogRendezvousHash {
  sorted_hashes: Vec<HashedId>,
}

impl LogRendezvousHash {
  pub fn new(ids: Vec<String>) -> LogRendezvousHash {
    let mut sorted_hashes = Vec::with_capacity(ids.len());
    for id in ids {
      sorted_hashes.push(HashedId::new(id));
    }
    sorted_hashes.sort_by(|a, b| a.hash.cmp(&b.hash));
    LogRendezvousHash { sorted_hashes }
  }

  /// Replaces the prior set of ids with the new set. TODO: Skip re-hashing ids that already exist
  pub fn update(self: &mut LogRendezvousHash, ids: Vec<String>) {
    let mut sorted_hashes = Vec::with_capacity(ids.len());
    for id in ids {
      sorted_hashes.push(HashedId::new(id));
    }
    sorted_hashes.sort_by(|a, b| a.hash.cmp(&b.hash));
    self.sorted_hashes = sorted_hashes;
  }

  pub fn get_leader_id(&self) -> &str {
    if self.sorted_hashes.len() == 0 {
      return "";
    }
    let mut last_idx = self.sorted_hashes.len() - 1;
    while last_idx > 0 && !self.sorted_hashes[last_idx].is_up {
      last_idx = last_idx - 1;
    }
    &self.sorted_hashes[last_idx].id
  }

  pub fn get_mut_nodes(self: &mut LogRendezvousHash) -> &mut Vec<HashedId> {
    &mut self.sorted_hashes
  }

  pub fn _get_primary_node_id(&self, key: &str) -> &str {
    self.get_assigned_nodes_id(key)[0]
  }

  pub fn get_assigned_nodes_id(&self, key: &str) -> Vec<&str> {
    let top = cmp::min(3, self.sorted_hashes.len());
    let mut idx = self.get_idx_for_key(key);
    let mut ids: Vec<&str> = Vec::new();
    for _ in 0..top {
      ids.push(&self.sorted_hashes[idx].id);
      idx = (idx + 1) % self.sorted_hashes.len();
    }
    ids
  }

  // Runs a binary search for the record whose hash is closest to the key hash without
  // going over. If none are found, the *last* record in the list is returned as it wraps around.
  fn get_idx_for_key(&self, key: &str) -> usize {
    let mut key_hasher = XxHash64::with_seed(0xa1a2);
    key_hasher.write(key.as_bytes());
    let key_hash = key_hasher.finish();
    let idx = match self
      .sorted_hashes
      .binary_search_by(|a| a.hash.cmp(&key_hash))
    {
      Ok(res) => res,
      // All were too large, implies last (which wraps around) owns it
      Err(_) => self.sorted_hashes.len() - 1,
    };
    idx
  }
}

pub struct ControlPort {
  lrh: LogRendezvousHash,
  client: Client<HttpsConnector<HttpConnector>>,
  // TODO: Once the crazy type info of the server can be figured out, we can attach it to this
  // struct and then make it possible to wind down the control port server
  // server: &'a dyn Service<std::convert::Infallible>,
  vms: HashMap<String, VMMetadata>, // All VMs in the cluster. String is private IP
  self_vm: Option<VMMetadata>,      // This VM. Not set on initialization
  region_vms: HashMap<String, VMMetadata>, // VMs in the same cloud and region. String is private IP
  vms_up: HashMap<String, bool>,    // VMs by private IP versus health status
}

async fn control_port(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let cluster_secret = CLUSTER_SECRET.get().unwrap();
  if cluster_secret.is_some() && !req.headers().contains_key(cluster_secret.as_ref().unwrap()) {
    // If this control port is guarded by a secret string, make sure there's a header with that
    // secret as the key (we don't care about the value) and abort otherwise
    return Ok(Response::builder().status(500).body("fail".into()).unwrap());
  }
  match req.uri().path() {
    "/health" => Ok(Response::builder().status(200).body("ok".into()).unwrap()),
    "/clusterHealth" => handle_cluster_health(),
    "/start" => handle_start(req).await,
    _ => Ok(Response::builder().status(404).body("fail".into()).unwrap()),
  }
}

fn handle_cluster_health() -> Result<Response<Body>, Infallible> {
  if TcpStream::connect("127.0.0.1:443").is_err() {
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

async fn handle_start(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  // Receive POST and save daemon properties
  match get_daemon_props(req).await {
    Ok(_) => Ok(Response::builder().status(200).body("ok".into()).unwrap()),
    Err(err) => {
      error!(DaemonStartFailed, "{:?}", err).await;
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn get_daemon_props(req: Request<Body>) -> DaemonResult<()> {
  let bytes = body::to_bytes(req.into_body()).await?;
  let body: DaemonProperties = serde_json::from_slice(&bytes).unwrap();
  maybe_dump_files(&body).await?;
  DAEMON_PROPS.set(body).unwrap();
  Ok(())
}

async fn maybe_dump_files(daemon_props: &DaemonProperties) -> DaemonResult<()> {
  let pwd = env::current_dir();
  match pwd {
    Ok(pwd) => {
      for (file_name, content) in &daemon_props.filesB64 {
        write_b64_file(&pwd, file_name, content)?;
      }
    }
    Err(err) => {
      let err = format!("{:?}", err);
      return Err(err.into());
    }
  }
  Ok(())
}

fn write_b64_file(pwd: &PathBuf, file_name: &str, content: &str) -> io::Result<()> {
  write(
    format!("{}/{}", pwd.display(), file_name),
    base64::decode(content).unwrap(),
  )
}

// TODO: Revive once rustls supports IP addresses
/*mod naive {
  use rustls;

  pub struct TLS {}

  impl rustls::ServerCertVerifier for TLS {
    fn verify_server_cert(
      &self,
      _roots: &rustls::RootCertStore,
      _presented_certs: &[rustls::Certificate],
      _dns_name: tokio_rustls::webpki::DNSNameRef,
      _ocsp_response: &[u8],
    ) -> Result<rustls::ServerCertVerified, rustls::TLSError> {
      Ok(rustls::ServerCertVerified::assertion())
    }
  }
}*/

impl ControlPort {
  pub async fn start() -> ControlPort {
    let pwd = env::current_dir();
    match pwd {
      Ok(pwd) => {
        let priv_key = read(format!("{}/key.pem", pwd.display()));
        let cert = read(format!("{}/certificate.pem", pwd.display()));
        if let (Ok(priv_key), Ok(cert)) = (priv_key, cert) {
          // TODO: Make this not a side-effect
          make_server!(
            HttpType::HTTPS(HttpsConfig {
              port: 4142, // 4 = A, 1 = L, 2 = N (sideways) => ALAN
              priv_key: String::from_utf8(priv_key).unwrap(),
              cert: String::from_utf8(cert).unwrap(),
            }),
            control_port
          );
          let mut tls = SslConnector::builder(SslMethod::tls_client()).unwrap();
          tls.set_verify(SslVerifyMode::NONE);
          /*let mut tls = ClientConfig::new();
          tls
            .dangerous()
            .set_certificate_verifier(Arc::new(naive::TLS {}));*/
          let mut http_connector = HttpConnector::new();
          http_connector.enforce_http(false);

          // This works because we only construct the control port once
          let mut https = HttpsConnector::with_connector(http_connector, tls).unwrap();
          https.set_callback(|cc, _| {
            cc.set_use_server_name_indication(false);
            Ok(())
          });
          let client = Client::builder().build::<_, Body>(https);
          //let client = Client::builder().build::<_, Body>(HttpsConnector::from((http_connector, tls)));
          NAIVE_CLIENT.set(client).unwrap();
          // Make a second client. TODO: Share this? Or split into a naive-client generator function?
          let mut tls = SslConnector::builder(SslMethod::tls_client()).unwrap();
          tls.set_verify(SslVerifyMode::NONE);
          /*let mut tls = ClientConfig::new();
          tls
            .dangerous()
            .set_certificate_verifier(Arc::new(naive::TLS {}));*/
          let mut http_connector = HttpConnector::new();
          http_connector.enforce_http(false);
          let mut https = HttpsConnector::with_connector(http_connector, tls).unwrap();
          https.set_callback(|cc, _| {
            cc.set_use_server_name_indication(false);
            Ok(())
          });
          let client = Client::builder().build::<_, Body>(https);
          //let client = Client::builder().build::<_, Body>(HttpsConnector::from((http_connector, tls)));

          ControlPort {
            lrh: LogRendezvousHash::new(vec![]),
            client,
            vms: HashMap::new(),
            self_vm: None,
            region_vms: HashMap::new(),
            vms_up: HashMap::new(),
          }
        } else {
          let err = "Failed getting ssl certificate or key";
          error!(NoSSLCert, "{}", err).await;
          std::process::exit(1);
        }
      }
      Err(err) => {
        let err = format!("{:?}", err);
        error!(CtrlPortStartFailed, "{:?}", err).await;
        std::process::exit(1);
      }
    }
  }

  pub async fn update_vms(self: &mut ControlPort, self_ip: &str, vms: Vec<VMMetadata>) {
    let ips = vms
      .iter()
      .map(|vm| vm.private_ip_addr.to_string())
      .collect();
    let self_vm_vec: Vec<&VMMetadata> = vms
      .iter()
      .filter(|vm| vm.private_ip_addr == self_ip)
      .collect();
    if self_vm_vec.len() == 0 {
      error!(
        NoDnsPrivateIp,
        "Failed to find self in cluster. Maybe I am being shut down?"
      )
      .await;
      // TODO: Should this error propagate up to the stats loop or no?
      return;
    } else if self_vm_vec.len() > 1 {
      // This hopefully never happens, but if it does, we need to change the daemon initialization
      error!(
        DuplicateDnsPrivateIp,
        "Private IP address collision detected! I don't know who I really am!"
      )
      .await;
      // TODO: Should this error propagate up to the stats loop or no?
      return;
    }
    let self_vm = self_vm_vec[0].clone();
    let mut region_vms = HashMap::new();
    vms
      .iter()
      .filter(|vm| vm.cloud == self_vm.cloud && vm.region == self_vm.region)
      .for_each(|vm| {
        region_vms.insert(vm.private_ip_addr.clone(), vm.clone());
      });
    let mut all_vms = HashMap::new();
    vms.iter().for_each(|vm| {
      all_vms.insert(vm.private_ip_addr.clone(), vm.clone());
    });
    let mut other_region_ips: Vec<String> = region_vms
      .keys()
      .filter(|ip| ip.as_str() != self_ip)
      .filter(|ip| *self.vms_up.get(ip.clone()).unwrap_or(&false))
      .map(|ip| ip.clone())
      .collect();
    let region_ips = Arc::clone(&REGION_VMS);
    let mut region_ips_mut = region_ips.write().unwrap();
    region_ips_mut.clear();
    region_ips_mut.append(&mut other_region_ips);
    drop(region_ips_mut);
    drop(region_ips);
    self.vms = all_vms;
    self.self_vm = Some(self_vm);
    self.region_vms = region_vms;
    self.lrh.update(ips);
  }

  pub fn is_leader(self: &ControlPort) -> bool {
    match &self.self_vm {
      Some(self_vm) => self.lrh.get_leader_id() == self_vm.private_ip_addr,
      None => false,
    }
  }

  pub fn get_leader(self: &ControlPort) -> Option<&VMMetadata> {
    self.vms.get(self.lrh.get_leader_id())
  }

  pub async fn check_cluster_health(self: &mut ControlPort) {
    let cluster_secret = CLUSTER_SECRET.get().unwrap().as_ref().unwrap().clone();
    let mut health = vec![];
    let nodes = self.lrh.get_mut_nodes();
    for node in nodes.iter() {
      let mut req = Request::builder()
        .method("GET")
        .uri(format!("https://{}:4142/clusterHealth", node.id));
      req = req.header(cluster_secret.as_str(), "true");
      health.push(self.client.request(req.body(Body::empty()).unwrap()));
    }
    let health_res = join_all(health).await;
    for (i, res) in health_res.iter().enumerate() {
      match res {
        Err(_) => {
          nodes[i].is_up = false;
          self.vms_up.insert(nodes[i].id.clone(), false);
        }
        Ok(res) => {
          let is_up = res.status().as_u16() == 200;
          nodes[i].is_up = is_up;
          self.vms_up.insert(nodes[i].id.clone(), is_up);
        }
      }
    }
  }
}
