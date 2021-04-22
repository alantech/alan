use std::cmp;
use std::convert::Infallible;
use std::hash::Hasher;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;

use futures::future::join_all;
use hyper::{
  client::{Client, HttpConnector},
  Body, Request, Response,
};
use hyper_rustls::HttpsConnector;
use rustls::ClientConfig;
use twox_hash::XxHash64;

use crate::daemon::daemon::CLUSTER_SECRET;
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
  pub async fn start(priv_key_b64: &str, cert_b64: &str) -> ControlPort {
    // TODO: Make this not a side-effect
    make_server!(
      HttpType::HTTPS(HttpsConfig {
        port: 4142, // 4 = A, 1 = L, 2 = N (sideways) => ALAN
        priv_key_b64: priv_key_b64.to_string(),
        cert_b64: cert_b64.to_string(),
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
        .uri(format!("https://{}:4142/", node.id));
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
