use std::convert::Infallible;
use std::env;
use std::error::Error;
use std::fs::{read, write};
use std::hash::Hasher;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;

use anycloud::logger::ErrorType;
use anycloud::error;
use futures::future::join_all;
use hyper::{
  body,
  client::{Client, HttpConnector},
  Body, Request, Response,
};
use hyper_rustls::HttpsConnector;
use rustls::ClientConfig;
use twox_hash::XxHash64;

use crate::daemon::daemon::{DaemonProperties, CLUSTER_SECRET, DAEMON_PROPS};
use crate::make_server;
use crate::vm::http::{HttpType, HttpsConfig};

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
    let mut last_idx = self.sorted_hashes.len() - 1;
    while last_idx > 0 && !self.sorted_hashes[last_idx].is_up {
      last_idx = last_idx - 1;
    }
    &self.sorted_hashes[last_idx].id
  }

  pub fn get_mut_nodes(self: &mut LogRendezvousHash) -> &mut Vec<HashedId> {
    &mut self.sorted_hashes
  }

  // Runs a binary search for the record whose hash is closest to the key hash without
  // going over. If none are found, the *last* record in the list is returned as it wraps around.
  pub fn _get_id_for_key(&self, key: &str) -> &str {
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
    &self.sorted_hashes[idx].id
  }
}

pub struct ControlPort {
  lrh: LogRendezvousHash,
  client: Client<HttpsConnector<HttpConnector>>,
  // TODO: Once the crazy type info of the server can be figured out, we can attach it to this
  // struct and then make it possible to wind down the control port server
  // server: &'a dyn Service<std::convert::Infallible>,
}

async fn control_port(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  let cluster_secret = CLUSTER_SECRET.get().unwrap();
  if cluster_secret.is_some() && !req.headers().contains_key(cluster_secret.as_ref().unwrap()) {
    // If this control port is guarded by a secret string, make sure there's a header with that
    // secret as the key (we don't care about the value) and abort otherwise
    return Ok(Response::builder().status(500).body("fail".into()).unwrap());
  }
  match req.uri().path() {
    "/ping" => Ok(Response::builder().status(200).body("pong".into()).unwrap()),
    "/health" => handle_health(),
    "/start" => handle_start(req).await,
    _ => Ok(Response::builder().status(404).body("fail".into()).unwrap()),
  }
}

fn handle_health() -> Result<Response<Body>, Infallible> {
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
      error!(ErrorType::DaemonStartFailed, "{:?}", err).await;
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    },
  }
}

async fn get_daemon_props(req: Request<Body>) -> Result<(), Box<dyn Error>> {
  let bytes = body::to_bytes(req.into_body()).await?;
  let body: DaemonProperties = serde_json::from_slice(&bytes).unwrap();
  let pwd = env::current_dir();
  match pwd {
    Ok(pwd) => {
      if let Some(dockerfile_b64) = &body.dockerfileB64 {
        write(
          format!("{}/Dockerfile", pwd.display()),
          base64::decode(dockerfile_b64).unwrap(),
        )?;
      }
      if let Some(app_tar_gz_b64) = &body.appTarGzB64 {
        write(
          format!("{}/app.tar.gz", pwd.display()),
          base64::decode(app_tar_gz_b64).unwrap(),
        )?;
      }
      if let Some(env_b64) = &body.envB64 {
        write(
          format!("{}/anycloud.env", pwd.display()),
          base64::decode(env_b64).unwrap(),
        )?;
      }
    }
    Err(err) => {
      let err = format!("{:?}", err);
      return Err(err.into());
    }
  }
  DAEMON_PROPS
    .set(DaemonProperties {
      clusterId: body.clusterId,
      agzB64: body.agzB64,
      deployToken: body.deployToken,
      domain: body.domain,
      dockerfileB64: body.dockerfileB64,
      appTarGzB64: body.appTarGzB64,
      envB64: body.envB64,
    })
    .unwrap();
  Ok(())
}

mod naive {
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
}

impl ControlPort {
  pub async fn start() -> ControlPort {
    let pwd = env::current_dir();
    match pwd {
      Ok(pwd) => {
        let priv_key_b64 = read(format!("{}/priv_key_b64", pwd.display()));
        let cert_b64 = read(format!("{}/cert_b64", pwd.display()));
        if let (Ok(priv_key_b64), Ok(cert_b64)) = (priv_key_b64, cert_b64) {
          // TODO: Make this not a side-effect
          make_server!(
            HttpType::HTTPS(HttpsConfig {
              port: 4142, // 4 = A, 1 = L, 2 = N (sideways) => ALAN
              priv_key_b64: String::from_utf8(priv_key_b64).unwrap(),
              cert_b64: String::from_utf8(cert_b64).unwrap(),
            }),
            control_port
          );
          let mut tls = ClientConfig::new();
          tls
            .dangerous()
            .set_certificate_verifier(Arc::new(naive::TLS {}));
          let mut http_connector = HttpConnector::new();
          http_connector.enforce_http(false);

          ControlPort {
            lrh: LogRendezvousHash::new(vec![]),
            client: Client::builder().build::<_, Body>(HttpsConnector::from((http_connector, tls))),
          }
        } else {
          let err = "Failed getting ssl certificate or key";
          error!(ErrorType::CtrlPortStartFailed, "{}", err).await;
          panic!("{}", err);
        }
      }
      Err(err) => {
        let err = format!("{:?}", err);
        error!(ErrorType::CtrlPortStartFailed, "{:?}", err).await;
        panic!("{:?}", err);
      }
    }
  }

  pub fn update_ips(self: &mut ControlPort, ips: Vec<String>) {
    self.lrh.update(ips);
  }

  pub fn get_leader(self: &ControlPort) -> &str {
    self.lrh.get_leader_id()
  }

  pub async fn check_cluster_health(self: &mut ControlPort) {
    let cluster_secret = CLUSTER_SECRET.get().unwrap().as_ref().unwrap().clone();
    let mut health = vec![];
    let nodes = self.lrh.get_mut_nodes();
    for node in nodes.iter() {
      let mut req = Request::builder()
        .method("GET")
        .uri(format!("https://{}:4142/health", node.id));
      req = req.header(cluster_secret.as_str(), "true");
      health.push(self.client.request(req.body(Body::empty()).unwrap()));
    }
    let health_res = join_all(health).await;
    for (i, res) in health_res.iter().enumerate() {
      match res {
        Err(_) => {
          nodes[i].is_up = false;
        }
        Ok(res) => {
          nodes[i].is_up = res.status().as_u16() == 200;
        }
      }
    }
  }
}
