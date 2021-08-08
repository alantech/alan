use std::cmp;
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use std::fs::{read, write};
use std::hash::Hasher;
use std::io;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::str;
use std::sync::Arc;

use futures::future::join_all;
use hyper::{
  body,
  client::{Client, HttpConnector},
  header::{HeaderName, HeaderValue},
  Body, Request, Response, StatusCode,
};
// TODO: Restore rustls once it can connect directly by IP address
//use hyper_rustls::HttpsConnector;
use hyper_openssl::HttpsConnector;
use once_cell::sync::OnceCell;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
//use rustls::ClientConfig;
use protobuf::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::oneshot::{self, Receiver, Sender};
use tokio::task;
use twox_hash::XxHash64;

use crate::daemon::daemon::{DaemonProperties, DaemonResult, CLUSTER_SECRET, DAEMON_PROPS};
use crate::daemon::dns::VMMetadata;
use crate::make_server;
use crate::vm::event::{BuiltInEvents, EventEmit, HandlerFragment};
use crate::vm::http::{HttpType, HttpsConfig};
use crate::vm::memory::{HandlerMemory, CLOSURE_ARG_MEM_START};
use crate::vm::opcode::{DS, REGION_VMS};
use crate::vm::protos;
use crate::vm::run::EVENT_TX;
use crate::vm::{VMError, VMResult};
use crate::{error, warn};

pub static NAIVE_CLIENT: OnceCell<Client<HttpsConnector<HttpConnector>>> = OnceCell::new();
pub static CONTROL_PORT_EXTENSIONS: OnceCell<bool> = OnceCell::new();

#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
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

  pub fn get_primary_node_id(&self, key: &str) -> &str {
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
      Err(res) => res, // This is actually the last index less than the hash, which is what we want
    } % self.sorted_hashes.len();
    idx
  }
}

#[derive(Clone, Debug)]
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
    "/datastore/getf" => handle_dsgetf(req).await, // TODO: How to better organize the datastore stuff?
    "/datastore/getv" => handle_dsgetv(req).await,
    "/datastore/getr" => handle_dsgetr(req).await,
    "/datastore/has" => handle_dshas(req).await,
    "/datastore/del" => handle_dsdel(req).await,
    "/datastore/setf" => handle_dssetf(req).await,
    "/datastore/setv" => handle_dssetv(req).await,
    "/datastore/keys" => handle_keys(),
    "/datastore/dsrrun" => handle_dsrrun(req).await,
    "/datastore/dsmrun" => handle_dsmrun(req).await,
    "/datastore/dsrwith" => handle_dsrwith(req).await,
    "/datastore/dsmwith" => handle_dsmwith(req).await,
    "/datastore/dsmonly" => handle_dsmonly(req).await,
    "/datastore/dswonly" => handle_dswonly(req).await,
    "/datastore/dsrclos" => handle_dsrclos(req).await,
    "/datastore/dsmclos" => handle_dsmclos(req).await,
    path => {
      if *CONTROL_PORT_EXTENSIONS.get().unwrap() && path.starts_with("/app/") {
        handle_extensions(req).await
      } else {
        Ok(Response::builder().status(404).body("fail".into()).unwrap())
      }
    }
  }
}

async fn handle_extensions(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match extension_listener(req).await {
    Ok(res) => Ok(res),
    Err(_) => Ok(Response::builder().status(404).body("fail".into()).unwrap()),
  }
}

async fn extension_listener(req: Request<Body>) -> VMResult<Response<Body>> {
  // Stolen from the `http_listener` in the opcodes with minor modifications. TODO: DRY this out
  // Create a new event handler memory to add to the event queue
  let mut event = HandlerMemory::new(None, 1)?;
  // Grab the method
  let method_str = req.method().to_string();
  let method = HandlerMemory::str_to_fractal(&method_str);
  // Grab the URL
  let orig_uri = req.uri().clone();
  let orig_query = match orig_uri.query() {
    Some(q) => format!("?{}", q),
    None => format!(""),
  };
  let url_str = format!("{}{}", orig_uri.path(), orig_query);
  //let url_str = req.uri().to_string();
  let url = HandlerMemory::str_to_fractal(&url_str);
  // Grab the headers
  let headers = req.headers();
  let mut headers_hm = HandlerMemory::new(None, headers.len() as i64)?;
  headers_hm.init_fractal(CLOSURE_ARG_MEM_START)?;
  for (i, (key, val)) in headers.iter().enumerate() {
    let key_str = key.as_str();
    // TODO: get rid of the potential panic here
    let val_str = val.to_str().unwrap();
    headers_hm.init_fractal(i as i64)?;
    headers_hm.push_fractal(i as i64, HandlerMemory::str_to_fractal(key_str))?;
    headers_hm.push_fractal(i as i64, HandlerMemory::str_to_fractal(val_str))?;
    headers_hm.push_register(CLOSURE_ARG_MEM_START, i as i64)?;
  }
  // Grab the body, if any
  let body_req = match hyper::body::to_bytes(req.into_body()).await {
    Ok(bytes) => bytes,
    // If we error out while getting the body, just close this listener out immediately
    Err(ee) => {
      return Ok(Response::new(
        format!("Connection terminated: {}", ee).into(),
      ));
    }
  };
  // TODO: get rid of the potential panic here
  let body_str = str::from_utf8(&body_req).unwrap().to_string();
  let body = HandlerMemory::str_to_fractal(&body_str);
  // Populate the event and emit it
  event.init_fractal(0)?;
  event.push_fractal(0, method)?;
  event.push_fractal(0, url)?;
  HandlerMemory::transfer(
    &headers_hm,
    CLOSURE_ARG_MEM_START,
    &mut event,
    CLOSURE_ARG_MEM_START,
  )?;
  event.push_register(0, CLOSURE_ARG_MEM_START)?;
  event.push_fractal(0, body)?;
  // Generate a threadsafe raw ptr to the tx of a watch channel
  // A ptr is unsafely created from the raw ptr in httpsend once the
  // user's code has completed and sends the new HandlerMemory so we
  // can resume execution of this HTTP request
  let (tx, rx): (Sender<Arc<HandlerMemory>>, Receiver<Arc<HandlerMemory>>) = oneshot::channel();
  let tx_ptr = Box::into_raw(Box::new(tx)) as i64;
  event.push_fixed(0, tx_ptr)?;
  let event_emit = EventEmit {
    id: i64::from(BuiltInEvents::CTRLPORT),
    payload: Some(event),
  };
  let event_tx = EVENT_TX.get().ok_or(VMError::ShutDown)?;
  let mut err_res = Response::new("Error synchronizing `send` for HTTP request".into());
  *err_res.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
  if event_tx.send(event_emit).is_err() {
    return Ok(err_res);
  }
  // Await HTTP response from the user code
  let response_hm = match rx.await {
    Ok(hm) => hm,
    Err(_) => {
      return Ok(err_res);
    }
  };
  // Get the status from the user response and begin building the response object
  let status = response_hm.read_fixed(0)? as u16;
  let mut res = Response::builder()
    .status(StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR));
  // Get the headers and populate the response object
  // TODO: figure out how to handle this potential panic
  let headers = res.headers_mut().unwrap();
  let header_hms = response_hm.read_fractal(1)?;
  for i in 0..header_hms.len() {
    let (h, _) = response_hm.read_from_fractal(&header_hms.clone(), i);
    let (key_hm, _) = response_hm.read_from_fractal(&h, 0);
    let (val_hm, _) = response_hm.read_from_fractal(&h, 1);
    let key = HandlerMemory::fractal_to_string(key_hm)?;
    let val = HandlerMemory::fractal_to_string(val_hm)?;
    // TODO: figure out how to handle this potential panic
    let name = HeaderName::from_bytes(key.as_bytes()).unwrap();
    // TODO: figure out how to handle this potential panic
    let value = HeaderValue::from_str(&val).unwrap();
    headers.insert(name, value);
  }
  // Get the body, populate the response object, and fire it out
  let body = HandlerMemory::fractal_to_string(response_hm.read_fractal(2)?)?;
  // TODO: figure out how to handle this potential panic
  Ok(res.body(body.into()).unwrap())
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

#[derive(Deserialize, Debug, Serialize)]
struct DSGet {
  pub nskey: String,
}

async fn handle_dsgetf(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dsgetf_inner(req).await {
    Ok(hand_mem) => {
      let mut out = vec![];
      hand_mem.to_pb().write_to_vec(&mut out).unwrap();
      Ok(Response::builder().status(200).body(out.into()).unwrap())
    }
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dsgetf_inner(req: Request<Body>) -> DaemonResult<Arc<HandlerMemory>> {
  // TODO: For now assume this was directed at the right node, later on add auto-forwarding logic
  let bytes = body::to_bytes(req.into_body()).await?;
  let body: DSGet = serde_json::from_slice(&bytes)?;
  let maybe_hm = DS.get(&body.nskey);
  let mut hand_mem = HandlerMemory::new(None, 1)?;
  hand_mem.init_fractal(0)?;
  hand_mem.push_fixed(0, if maybe_hm.is_some() { 1i64 } else { 0i64 })?;
  match maybe_hm {
    Some(hm) => hand_mem.push_fixed(0, hm.read_fixed(0)?),
    None => hand_mem.push_fractal(
      0,
      HandlerMemory::str_to_fractal("namespace-key pair not found"),
    ),
  }?;
  Ok(hand_mem)
}

async fn handle_dsgetv(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dsgetv_inner(req).await {
    Ok(hand_mem) => {
      let mut out = vec![];
      hand_mem.to_pb().write_to_vec(&mut out).unwrap();
      Ok(Response::builder().status(200).body(out.into()).unwrap())
    }
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dsgetv_inner(req: Request<Body>) -> DaemonResult<Arc<HandlerMemory>> {
  let bytes = body::to_bytes(req.into_body()).await?;
  let body: DSGet = serde_json::from_slice(&bytes)?;
  let maybe_hm = DS.get(&body.nskey);
  let mut hand_mem = HandlerMemory::new(None, 1)?;
  hand_mem.init_fractal(0)?;
  hand_mem.push_fixed(0, if maybe_hm.is_some() { 1i64 } else { 0i64 })?;
  match maybe_hm {
    Some(hm) => {
      HandlerMemory::transfer(&hm, 0, &mut hand_mem, CLOSURE_ARG_MEM_START)?;
      hand_mem.push_register(0, CLOSURE_ARG_MEM_START)?;
    }
    None => {
      hand_mem.push_fractal(
        0,
        HandlerMemory::str_to_fractal("namespace-key pair not found"),
      )?;
    }
  };
  Ok(hand_mem)
}

async fn handle_dsgetr(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dsgetr_inner(req).await {
    Ok(hand_mem) => {
      let mut out = vec![];
      hand_mem.to_pb().write_to_vec(&mut out).unwrap();
      Ok(Response::builder().status(200).body(out.into()).unwrap())
    }
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dsgetr_inner(req: Request<Body>) -> DaemonResult<Arc<HandlerMemory>> {
  let bytes = body::to_bytes(req.into_body()).await?;
  let body: DSGet = serde_json::from_slice(&bytes)?;
  let maybe_hm = DS.get(&body.nskey);
  match maybe_hm {
    Some(hm) => Ok(hm.clone()),
    None => Err(Box::new(VMError::Other(
      "namespace-key pair not found".to_string(),
    ))),
  }
}

async fn handle_dshas(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dshas_inner(req).await {
    Ok(has) => Ok(
      Response::builder()
        .status(200)
        .body(has.to_string().into())
        .unwrap(),
    ),
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dshas_inner(req: Request<Body>) -> DaemonResult<bool> {
  let bytes = body::to_bytes(req.into_body()).await?;
  let body: DSGet = serde_json::from_slice(&bytes)?;
  Ok(DS.contains_key(&body.nskey))
}

async fn handle_dsdel(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dsdel_inner(req).await {
    Ok(del) => Ok(
      Response::builder()
        .status(200)
        .body(del.to_string().into())
        .unwrap(),
    ),
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dsdel_inner(req: Request<Body>) -> DaemonResult<bool> {
  let bytes = body::to_bytes(req.into_body()).await?;
  let body: DSGet = serde_json::from_slice(&bytes)?;
  Ok(DS.remove(&body.nskey).is_some())
}

async fn handle_dssetf(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dssetf_inner(req).await {
    Ok(_) => Ok(Response::builder().status(200).body("true".into()).unwrap()),
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dssetf_inner(req: Request<Body>) -> DaemonResult<()> {
  let bytes = body::to_bytes(req.into_body()).await?;
  let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
  let hand_mem = HandlerMemory::from_pb(&pb)?;
  let nskey = HandlerMemory::fractal_to_string(hand_mem.read_fractal(0)?)?;
  let val = hand_mem.read_fixed(1)?;
  let mut hm = HandlerMemory::new(None, 1)?;
  hm.write_fixed(0, val)?;
  DS.insert(nskey, hm);
  Ok(())
}

async fn handle_dssetv(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dssetv_inner(req).await {
    Ok(_) => Ok(Response::builder().status(200).body("true".into()).unwrap()),
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dssetv_inner(req: Request<Body>) -> DaemonResult<()> {
  let bytes = body::to_bytes(req.into_body()).await?;
  let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
  let hand_mem = HandlerMemory::from_pb(&pb)?;
  let nskey = HandlerMemory::fractal_to_string(hand_mem.read_fractal(0)?)?;
  let mut hm = HandlerMemory::new(None, 1)?;
  HandlerMemory::transfer(&hand_mem, 1, &mut hm, 0)?;
  DS.insert(nskey, hm);
  Ok(())
}

fn handle_keys() -> Result<Response<Body>, Infallible> {
  let keys = DS
    .iter()
    .map(|kvs| kvs.key().clone())
    .collect::<Vec<String>>()
    .join("\n");
  Ok(Response::builder().status(200).body(keys.into()).unwrap())
}

async fn handle_dsrrun(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dsrrun_inner(req).await {
    Ok(hand_mem) => {
      let mut out = vec![];
      hand_mem.to_pb().write_to_vec(&mut out).unwrap();
      Ok(Response::builder().status(200).body(out.into()).unwrap())
    }
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dsrrun_inner(req: Request<Body>) -> DaemonResult<Arc<HandlerMemory>> {
  let headers = req.headers();
  let nskey = headers.get("nskey").map_or("N/A", |v| v.to_str().unwrap());
  let maybe_hm = DS.get(nskey);
  let subhandler_id = headers
    .get("subhandler_id")
    .map_or(0, |v| v.to_str().unwrap().parse().unwrap());
  let subhandler = HandlerFragment::new(subhandler_id, 0);
  let bytes = body::to_bytes(req.into_body()).await?;
  let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
  let mut hm = HandlerMemory::from_pb(&pb)?;
  let mut res_hm = HandlerMemory::new(None, 1)?;
  res_hm.init_fractal(0)?;
  match maybe_hm {
    Some(ds) => {
      HandlerMemory::transfer(&ds, 0, &mut hm, CLOSURE_ARG_MEM_START + 1)?;
      let hm = subhandler.run(hm).await?;
      res_hm.push_fixed(0, 1);
      if hm.addr_to_idxs_opt(CLOSURE_ARG_MEM_START).is_some() {
        // Guard against void functions
        HandlerMemory::transfer(&hm, CLOSURE_ARG_MEM_START, &mut res_hm, 1);
        res_hm.push_register(0, 1)?;
      }
    }
    None => {
      res_hm.push_fixed(0, 0);
      res_hm.push_fractal(
        0,
        HandlerMemory::str_to_fractal("namespace-key pair not found"),
      )?;
    }
  }
  Ok(res_hm)
}

async fn handle_dsmrun(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dsmrun_inner(req).await {
    Ok(hand_mem) => {
      let mut out = vec![];
      hand_mem.to_pb().write_to_vec(&mut out).unwrap();
      Ok(Response::builder().status(200).body(out.into()).unwrap())
    }
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dsmrun_inner(req: Request<Body>) -> DaemonResult<Arc<HandlerMemory>> {
  let headers = req.headers();
  let nskey = headers
    .get("nskey")
    .map_or("N/A", |v| v.to_str().unwrap())
    .to_string();
  let maybe_hm = DS.get(&nskey);
  let subhandler_id = headers
    .get("subhandler_id")
    .map_or(0, |v| v.to_str().unwrap().parse().unwrap());
  let subhandler = HandlerFragment::new(subhandler_id, 0);
  let bytes = body::to_bytes(req.into_body()).await?;
  let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
  let mut hm = HandlerMemory::from_pb(&pb)?;
  let mut res_hm = HandlerMemory::new(None, 1)?;
  res_hm.init_fractal(0)?;
  match maybe_hm {
    Some(ds) => {
      HandlerMemory::transfer(&ds, 0, &mut hm, CLOSURE_ARG_MEM_START + 1)?;
      let hm = subhandler.run(hm).await?;
      res_hm.push_fixed(0, 1);
      if hm.addr_to_idxs_opt(CLOSURE_ARG_MEM_START).is_some() {
        // Guard against void functions
        HandlerMemory::transfer(&hm, CLOSURE_ARG_MEM_START, &mut res_hm, 1);
        res_hm.push_register(0, 1)?;
      }
      // Also grab the mutation to the datastore value and re-insert it
      let mut newds = HandlerMemory::new(None, 1)?;
      HandlerMemory::transfer(&hm, CLOSURE_ARG_MEM_START + 1, &mut newds, 0)?;
      drop(ds);
      DS.insert(nskey, newds);
    }
    None => {
      res_hm.push_fixed(0, 0);
      res_hm.push_fractal(
        0,
        HandlerMemory::str_to_fractal("namespace-key pair not found"),
      )?;
    }
  }
  Ok(res_hm)
}

async fn handle_dsrwith(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dsrwith_inner(req).await {
    Ok(hand_mem) => {
      let mut out = vec![];
      hand_mem.to_pb().write_to_vec(&mut out).unwrap();
      Ok(Response::builder().status(200).body(out.into()).unwrap())
    }
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dsrwith_inner(req: Request<Body>) -> DaemonResult<Arc<HandlerMemory>> {
  let headers = req.headers();
  let nskey = headers.get("nskey").map_or("N/A", |v| v.to_str().unwrap());
  let maybe_hm = DS.get(nskey);
  let subhandler_id = headers
    .get("subhandler_id")
    .map_or(0, |v| v.to_str().unwrap().parse().unwrap());
  let subhandler = HandlerFragment::new(subhandler_id, 0);
  let bytes = body::to_bytes(req.into_body()).await?;
  let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
  let mut hm = HandlerMemory::from_pb(&pb)?;
  let mut res_hm = HandlerMemory::new(None, 1)?;
  res_hm.init_fractal(0)?;
  match maybe_hm {
    Some(ds) => {
      HandlerMemory::transfer(&ds, 0, &mut hm, CLOSURE_ARG_MEM_START + 1)?;
      let hm = subhandler.run(hm).await?;
      res_hm.push_fixed(0, 1);
      if hm.addr_to_idxs_opt(CLOSURE_ARG_MEM_START).is_some() {
        // Guard against void functions
        HandlerMemory::transfer(&hm, CLOSURE_ARG_MEM_START, &mut res_hm, 1);
        res_hm.push_register(0, 1)?;
      }
    }
    None => {
      res_hm.push_fixed(0, 0);
      res_hm.push_fractal(
        0,
        HandlerMemory::str_to_fractal("namespace-key pair not found"),
      )?;
    }
  }
  Ok(res_hm)
}

async fn handle_dsmwith(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dsmwith_inner(req).await {
    Ok(hand_mem) => {
      let mut out = vec![];
      hand_mem.to_pb().write_to_vec(&mut out).unwrap();
      Ok(Response::builder().status(200).body(out.into()).unwrap())
    }
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dsmwith_inner(req: Request<Body>) -> DaemonResult<Arc<HandlerMemory>> {
  let headers = req.headers();
  let nskey = headers
    .get("nskey")
    .map_or("N/A", |v| v.to_str().unwrap())
    .to_string();
  let maybe_hm = DS.get(&nskey);
  let subhandler_id = headers
    .get("subhandler_id")
    .map_or(0, |v| v.to_str().unwrap().parse().unwrap());
  let subhandler = HandlerFragment::new(subhandler_id, 0);
  let bytes = body::to_bytes(req.into_body()).await?;
  let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
  let mut hm = HandlerMemory::from_pb(&pb)?;
  let mut res_hm = HandlerMemory::new(None, 2)?;
  res_hm.init_fractal(0)?;
  match maybe_hm {
    Some(ds) => {
      HandlerMemory::transfer(&ds, 0, &mut hm, CLOSURE_ARG_MEM_START + 1)?;
      let hm = subhandler.run(hm).await?;
      res_hm.push_fixed(0, 1);
      if hm.addr_to_idxs_opt(CLOSURE_ARG_MEM_START).is_some() {
        // Guard against void functions
        HandlerMemory::transfer(&hm, CLOSURE_ARG_MEM_START, &mut res_hm, 1);
        res_hm.push_register(0, 1)?;
      }
      // Also grab the mutation to the datastore value and re-insert it
      let mut newds = HandlerMemory::new(None, 1)?;
      HandlerMemory::transfer(&hm, CLOSURE_ARG_MEM_START + 1, &mut newds, 0)?;
      drop(ds);
      DS.insert(nskey, newds);
    }
    None => {
      res_hm.push_fixed(0, 0);
      res_hm.push_fractal(
        0,
        HandlerMemory::str_to_fractal("namespace-key pair not found"),
      )?;
    }
  }
  Ok(res_hm)
}

async fn handle_dsmonly(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dsmonly_inner(req).await {
    Ok(_) => Ok(Response::builder().status(200).body("ok".into()).unwrap()),
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dsmonly_inner(req: Request<Body>) -> DaemonResult<()> {
  let headers = req.headers();
  let nskey = headers
    .get("nskey")
    .map_or("N/A", |v| v.to_str().unwrap())
    .to_string();
  let maybe_hm = DS.get(&nskey);
  let subhandler_id = headers
    .get("subhandler_id")
    .map_or(0, |v| v.to_str().unwrap().parse().unwrap());
  let subhandler = HandlerFragment::new(subhandler_id, 0);
  let bytes = body::to_bytes(req.into_body()).await?;
  let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
  let mut hm = HandlerMemory::from_pb(&pb)?;
  match maybe_hm {
    Some(ds) => {
      HandlerMemory::transfer(&ds, 0, &mut hm, CLOSURE_ARG_MEM_START + 1)?;
      let hm = subhandler.run(hm).await?;
      // Also grab the mutation to the datastore value and re-insert it
      let mut newds = HandlerMemory::new(None, 1)?;
      HandlerMemory::transfer(&hm, CLOSURE_ARG_MEM_START + 1, &mut newds, 0)?;
      drop(ds);
      DS.insert(nskey, newds);
    }
    None => {
      // Do nothing
    }
  }
  Ok(())
}

async fn handle_dswonly(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dswonly_inner(req).await {
    Ok(_) => Ok(Response::builder().status(200).body("ok".into()).unwrap()),
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dswonly_inner(req: Request<Body>) -> DaemonResult<()> {
  let headers = req.headers();
  let nskey = headers
    .get("nskey")
    .map_or("N/A", |v| v.to_str().unwrap())
    .to_string();
  let maybe_hm = DS.get(&nskey);
  let subhandler_id = headers
    .get("subhandler_id")
    .map_or(0, |v| v.to_str().unwrap().parse().unwrap());
  let subhandler = HandlerFragment::new(subhandler_id, 0);
  let bytes = body::to_bytes(req.into_body()).await?;
  let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
  let mut hm = HandlerMemory::from_pb(&pb)?;
  match maybe_hm {
    Some(ds) => {
      HandlerMemory::transfer(&ds, 0, &mut hm, CLOSURE_ARG_MEM_START + 1)?;
      let hm = subhandler.run(hm).await?;
      // Also grab the mutation to the datastore value and re-insert it
      let mut newds = HandlerMemory::new(None, 1)?;
      HandlerMemory::transfer(&hm, CLOSURE_ARG_MEM_START + 1, &mut newds, 0)?;
      drop(ds);
      DS.insert(nskey, newds);
    }
    None => {
      // Do nothing
    }
  }
  Ok(())
}

async fn handle_dsrclos(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dsrclos_inner(req).await {
    Ok(hand_mem) => {
      let mut out = vec![];
      hand_mem.to_pb().write_to_vec(&mut out).unwrap();
      Ok(Response::builder().status(200).body(out.into()).unwrap())
    }
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dsrclos_inner(req: Request<Body>) -> DaemonResult<Arc<HandlerMemory>> {
  let headers = req.headers();
  let nskey = headers.get("nskey").map_or("N/A", |v| v.to_str().unwrap());
  let maybe_hm = DS.get(nskey);
  let subhandler_id = headers
    .get("subhandler_id")
    .map_or(0, |v| v.to_str().unwrap().parse().unwrap());
  let ret_addr = headers
    .get("ret_addr")
    .map_or(0, |v| v.to_str().unwrap().parse().unwrap());
  let subhandler = HandlerFragment::new(subhandler_id, 0);
  let bytes = body::to_bytes(req.into_body()).await?;
  let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
  let mut hand_mem = HandlerMemory::from_pb(&pb)?;
  hand_mem.init_fractal(ret_addr)?;
  match maybe_hm {
    Some(ds) => {
      let mut hm = HandlerMemory::fork(hand_mem.clone())?; // TODO: This clone is terrible
      HandlerMemory::transfer(&ds, 0, &mut hm, CLOSURE_ARG_MEM_START + 1)?;
      let hm = subhandler.run(hm).await?;
      let hm = hm.drop_parent()?;
      hand_mem.join(hm)?;
      hand_mem.push_fixed(ret_addr, 1i64)?;
      hand_mem.push_register(ret_addr, CLOSURE_ARG_MEM_START)?;
    }
    None => {
      hand_mem.push_fixed(ret_addr, 0)?;
      hand_mem.push_fractal(
        ret_addr,
        HandlerMemory::str_to_fractal("namespace-key pair not found"),
      )?;
    }
  }
  Ok(hand_mem)
}

async fn handle_dsmclos(req: Request<Body>) -> Result<Response<Body>, Infallible> {
  match dsmclos_inner(req).await {
    Ok(hand_mem) => {
      let mut out = vec![];
      hand_mem.to_pb().write_to_vec(&mut out).unwrap();
      Ok(Response::builder().status(200).body(out.into()).unwrap())
    }
    Err(err) => {
      // TODO: What error message here? Also should this also be a valid HM out of here?
      eprintln!("{:?}", err);
      Ok(Response::builder().status(500).body("fail".into()).unwrap())
    }
  }
}

async fn dsmclos_inner(req: Request<Body>) -> DaemonResult<Arc<HandlerMemory>> {
  eprintln!("dmclos_inner");
  let headers = req.headers();
  eprintln!("1");
  let nskey = headers
    .get("nskey")
    .map_or("N/A", |v| v.to_str().unwrap())
    .to_string();
  eprintln!("2");
  let maybe_hm = DS.get(&nskey);
  eprintln!("3");
  let subhandler_id = headers
    .get("subhandler_id")
    .map_or(0, |v| v.to_str().unwrap().parse().unwrap());
  eprintln!("4");
  let ret_addr = headers
    .get("ret_addr")
    .map_or(0, |v| v.to_str().unwrap().parse().unwrap());
  eprintln!("5");
  let subhandler = HandlerFragment::new(subhandler_id, 0);
  eprintln!("6");
  let bytes = body::to_bytes(req.into_body()).await?;
  eprintln!("7");
  let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
  eprintln!("8");
  let mut hand_mem = HandlerMemory::from_pb(&pb)?;
  eprintln!("9");
  hand_mem.init_fractal(ret_addr)?;
  eprintln!("10");
  match maybe_hm {
    Some(ds) => {
      let mut hm = HandlerMemory::fork(hand_mem.clone())?; // TODO: This clone is terrible
      eprintln!("a");
      HandlerMemory::transfer(&ds, 0, &mut hm, CLOSURE_ARG_MEM_START + 1)?;
      eprintln!("b");
      let hm = subhandler.run(hm).await?;
      eprintln!("c");
      // Also grab the mutation to the datastore value and re-insert it
      let mut newds = HandlerMemory::new(None, 1)?;
      eprintln!("d");
      HandlerMemory::transfer(&hm, CLOSURE_ARG_MEM_START + 1, &mut newds, 0)?;
      eprintln!("e");
      drop(ds);
      eprintln!("f");
      DS.insert(nskey, newds);
      eprintln!("g");
      let hm = hm.drop_parent()?;
      eprintln!("h");
      hand_mem.join(hm)?;
      eprintln!("i");
      hand_mem.push_fixed(ret_addr, 1i64)?;
      eprintln!("j");
      hand_mem.push_register(ret_addr, CLOSURE_ARG_MEM_START)?;
      eprintln!("k");
    }
    None => {
      eprintln!("hwat...");
      hand_mem.push_fixed(ret_addr, 0)?;
      hand_mem.push_fractal(
        ret_addr,
        HandlerMemory::str_to_fractal("namespace-key pair not found"),
      )?;
    }
  }
  Ok(hand_mem)
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
    let ips: Vec<String> = vms
      .iter()
      .map(|vm| vm.private_ip_addr.to_string())
      .collect();
    // Detect changes and exit early if nothing has changed
    let changed = ips.len() != self.vms.len() || ips.iter().any(|ip| !self.vms.contains_key(ip));
    if !changed {
      return;
    }
    let self_vm_vec: Vec<&VMMetadata> = vms
      .iter()
      .filter(|vm| vm.private_ip_addr == self_ip)
      .collect();
    if self_vm_vec.len() == 0 {
      warn!(
        NoDnsPrivateIp,
        "Failed to find self in cluster. Maybe I am being shut down or initialize?"
      );
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
    {
      // WTF, Rust? Why is `drop(var)` not good enough when there's an `await` later on?
      let region_ips = Arc::clone(&REGION_VMS);
      let mut region_ips_mut = region_ips.write().unwrap();
      region_ips_mut.clear();
      region_ips_mut.append(&mut other_region_ips);
    }
    self.vms = all_vms;
    self.self_vm = Some(self_vm);
    self.region_vms = region_vms;
    self.lrh.update(ips);
    self.rebalance_data().await;
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
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
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
    let mut health_change = false;
    for (i, res) in health_res.iter().enumerate() {
      let id = nodes[i].id.clone();
      match res {
        Err(_) => {
          if !self.vms_up.contains_key(&id) || *self.vms_up.get(&id).unwrap() == true {
            health_change = true;
          }
          nodes[i].is_up = false;
          self.vms_up.insert(id, false);
        }
        Ok(res) => {
          let is_up = res.status().as_u16() == 200;
          if !self.vms_up.contains_key(&id) || *self.vms_up.get(&id).unwrap() != is_up {
            health_change = true;
          }
          nodes[i].is_up = is_up;
          self.vms_up.insert(id, is_up);
        }
      }
    }
    if health_change {
      self.rebalance_data().await;
    }
  }

  pub fn is_up(self: &mut ControlPort) -> bool {
    match &self.self_vm {
      Some(self_vm) => match self.vms_up.get(&self_vm.private_ip_addr) {
        Some(s) => *s,
        None => false,
      },
      None => false,
    }
  }

  pub fn get_vm_for_key(self: &ControlPort, key: &str) -> &VMMetadata {
    &self.vms[self.lrh.get_primary_node_id(key)]
  }

  pub fn get_vms_for_key(self: &ControlPort, key: &str) -> Vec<&VMMetadata> {
    self
      .lrh
      .get_assigned_nodes_id(key)
      .into_iter()
      .map(|priv_ip_addr| &self.vms[&priv_ip_addr.to_string()])
      .collect()
  }

  pub fn get_closest_vm_for_key(self: &ControlPort, key: &str) -> (&VMMetadata, bool) {
    let vms = self.get_vms_for_key(key);
    let mut close_vms = vms
      .into_iter()
      .filter(|vm| self.region_vms.contains_key(&vm.private_ip_addr));
    match close_vms.next() {
      Some(close_vm) => (&close_vm, true),
      // Nothing is close, just go with the primary node
      None => (self.get_vm_for_key(key), false),
    }
  }

  pub fn is_key_owner(self: &ControlPort, key: &str) -> bool {
    match &self.self_vm {
      Some(my) => self.get_vm_for_key(key).private_ip_addr == my.private_ip_addr,
      None => false,
    }
  }

  pub async fn dsgetf(self: &ControlPort, key: &str) -> Option<Arc<HandlerMemory>> {
    self.dsgetf_inner(key).await.ok()
  }

  async fn dsgetf_inner(self: &ControlPort, key: &str) -> DaemonResult<Arc<HandlerMemory>> {
    let (vm, is_close) = self.get_closest_vm_for_key(key);
    let url = if is_close {
      format!("https://{}:4142/datastore/getf", vm.private_ip_addr)
    } else {
      format!("https://{}:4142/datastore/getf", vm.public_ip_addr)
    };
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req_obj = req.body(Body::from(
      json!(DSGet {
        nskey: key.to_string()
      })
      .to_string(),
    ))?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
    Ok(HandlerMemory::from_pb(&pb)?)
  }

  pub async fn dsgetv(self: &ControlPort, key: &str) -> Option<Arc<HandlerMemory>> {
    self.dsgetv_inner(key).await.ok()
  }

  async fn dsgetv_inner(self: &ControlPort, key: &str) -> DaemonResult<Arc<HandlerMemory>> {
    let (vm, is_close) = self.get_closest_vm_for_key(key);
    let url = if is_close {
      format!("https://{}:4142/datastore/getv", vm.private_ip_addr)
    } else {
      format!("https://{}:4142/datastore/getv", vm.public_ip_addr)
    };
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req_obj = req.body(Body::from(
      json!(DSGet {
        nskey: key.to_string()
      })
      .to_string(),
    ))?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
    Ok(HandlerMemory::from_pb(&pb)?)
  }

  pub async fn dshas(self: &ControlPort, key: &str) -> bool {
    self.dshas_inner(key).await.unwrap_or(false)
  }

  async fn dshas_inner(self: &ControlPort, key: &str) -> DaemonResult<bool> {
    let (vm, is_close) = self.get_closest_vm_for_key(key);
    let url = if is_close {
      format!("https://{}:4142/datastore/has", vm.private_ip_addr)
    } else {
      format!("https://{}:4142/datastore/has", vm.public_ip_addr)
    };
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req_obj = req.body(Body::from(
      json!(DSGet {
        nskey: key.to_string()
      })
      .to_string(),
    ))?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    Ok(std::str::from_utf8(&bytes)? == "true")
  }

  pub async fn dsdel(self: &ControlPort, key: &str) -> bool {
    let vms = self.get_vms_for_key(key);
    let urls: Vec<String> = vms
      .into_iter()
      .map(|vm| format!("https://{}:4142/datastore/del", vm.public_ip_addr))
      .collect();
    let calls = urls.into_iter().map(|url| self.dsdel_inner(url, key));
    let reses = join_all(calls).await;
    *reses[0].as_ref().unwrap_or(&false)
  }

  async fn dsdel_inner(self: &ControlPort, url: String, key: &str) -> DaemonResult<bool> {
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req_obj = req.body(Body::from(
      json!(DSGet {
        nskey: key.to_string()
      })
      .to_string(),
    ))?;
    let mut res = self.client.request(req_obj).await?;
    // TODO: How to handle if the various nodes are out-of-sync
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    Ok(std::str::from_utf8(&bytes)? == "true")
  }

  pub async fn dssetf(self: &ControlPort, key: &str, val: &Arc<HandlerMemory>) -> bool {
    let vms = self.get_vms_for_key(key);
    let urls: Vec<String> = vms
      .into_iter()
      .map(|vm| format!("https://{}:4142/datastore/setf", vm.public_ip_addr))
      .collect();
    let calls = urls.into_iter().map(|url| self.dssetf_inner(url, key, val));
    let reses = join_all(calls).await;
    *reses[0].as_ref().unwrap_or(&false)
  }

  async fn dssetf_inner(
    self: &ControlPort,
    url: String,
    key: &str,
    val: &Arc<HandlerMemory>,
  ) -> DaemonResult<bool> {
    let mut hm = HandlerMemory::new(None, 1)?;
    hm.write_fractal(0, &HandlerMemory::str_to_fractal(key))?;
    HandlerMemory::transfer(val, 0, &mut hm, 1)?;
    let mut out = vec![];
    hm.to_pb().write_to_vec(&mut out).unwrap();
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req_obj = req.body(Body::from(out))?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    Ok(std::str::from_utf8(&bytes)? == "true")
  }

  pub async fn dssetv(self: &ControlPort, key: &str, val: &Arc<HandlerMemory>) -> bool {
    let vms = self.get_vms_for_key(key);
    let urls: Vec<String> = vms
      .into_iter()
      .map(|vm| format!("https://{}:4142/datastore/setv", vm.public_ip_addr))
      .collect();
    let calls = urls.into_iter().map(|url| self.dssetv_inner(url, key, val));
    let reses = join_all(calls).await;
    *reses[0].as_ref().unwrap_or(&false)
  }

  async fn dssetv_inner(
    self: &ControlPort,
    url: String,
    key: &str,
    val: &Arc<HandlerMemory>,
  ) -> DaemonResult<bool> {
    let mut hm = HandlerMemory::new(None, 1)?;
    hm.write_fractal(0, &HandlerMemory::str_to_fractal(key))?;
    HandlerMemory::transfer(val, 0, &mut hm, 1)?;
    let mut out = vec![];
    hm.to_pb().write_to_vec(&mut out).unwrap();
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req_obj = req.body(Body::from(out))?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    Ok(std::str::from_utf8(&bytes)? == "true")
  }

  pub async fn dskeys(self: &ControlPort, ip: &str) -> (String, Vec<String>) {
    (
      ip.to_string(),
      self.dskeys_inner(ip).await.unwrap_or(Vec::new()),
    )
  }

  async fn dskeys_inner(self: &ControlPort, ip: &str) -> DaemonResult<Vec<String>> {
    let url = format!("https://{}:4142/datastore/keys", ip);
    let req = Request::builder().method("GET").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req_obj = req.body(Body::empty())?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    let key_str = std::str::from_utf8(&bytes)?;
    if key_str.len() > 0 {
      Ok(key_str.split("\n").map(|s| s.to_string()).collect())
    } else {
      Err(Box::new(VMError::Other(
        "No keys on remote node".to_string(),
      )))
    }
  }

  pub async fn dsrrun(
    self: &ControlPort,
    nskey: &str,
    subhandler_id: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> Arc<HandlerMemory> {
    let vm = self.get_vm_for_key(nskey);
    // TODO: Use private ip if possible
    let url = format!("https://{}:4142/datastore/dsrrun", vm.public_ip_addr);
    match self.dsrrun_inner(url, nskey, subhandler_id, hand_mem).await {
      Ok(hm) => hm,
      Err(_) => {
        let mut err_hm = HandlerMemory::new(None, 1).expect("what");
        err_hm
          .write_fractal(
            CLOSURE_ARG_MEM_START,
            &HandlerMemory::str_to_fractal("ERROR TODO"),
          )
          .expect("what");
        err_hm
      }
    }
  }

  async fn dsrrun_inner(
    self: &ControlPort,
    url: String,
    nskey: &str,
    subhandler_id: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> DaemonResult<Arc<HandlerMemory>> {
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req = req.header("nskey", nskey);
    let req = req.header("subhandler_id", format!("{}", subhandler_id));
    let orphan_hm = HandlerMemory::fork(hand_mem.clone())?; // TODO: This clone is a terrible idea
    let orphan_hm = orphan_hm.drop_parent()?;
    let mut out = vec![];
    orphan_hm.to_pb().write_to_vec(&mut out).unwrap();
    let req_obj = req.body(Body::from(out))?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
    Ok(HandlerMemory::from_pb(&pb)?)
  }

  pub async fn dsmrun(
    self: &ControlPort,
    nskey: &str,
    subhandler_id: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> Arc<HandlerMemory> {
    let vm = self.get_vm_for_key(nskey);
    // TODO: Use private ip if possible
    let url = format!("https://{}:4142/datastore/dsmrun", vm.public_ip_addr);
    match self.dsrrun_inner(url, nskey, subhandler_id, hand_mem).await {
      Ok(hm) => hm,
      Err(_) => {
        let mut err_hm = HandlerMemory::new(None, 1).expect("what");
        err_hm
          .write_fractal(
            CLOSURE_ARG_MEM_START,
            &HandlerMemory::str_to_fractal("ERROR TODO"),
          )
          .expect("what");
        err_hm
      }
    }
  }

  async fn dsmrun_inner(
    self: &ControlPort,
    url: String,
    nskey: &str,
    subhandler_id: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> DaemonResult<Arc<HandlerMemory>> {
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req = req.header("nskey", nskey);
    let req = req.header("subhandler_id", format!("{}", subhandler_id));
    let orphan_hm = HandlerMemory::fork(hand_mem.clone())?; // TODO: This clone is a terrible idea
    let orphan_hm = orphan_hm.drop_parent()?;
    let mut out = vec![];
    orphan_hm.to_pb().write_to_vec(&mut out).unwrap();
    let req_obj = req.body(Body::from(out))?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
    Ok(HandlerMemory::from_pb(&pb)?)
  }

  pub async fn dsrwith(
    self: &ControlPort,
    nskey: &str,
    with_addr: i64,
    subhandler_id: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> Arc<HandlerMemory> {
    let vm = self.get_vm_for_key(nskey);
    // TODO: Use private ip if possible
    let url = format!("https://{}:4142/datastore/dsrwith", vm.public_ip_addr);
    match self
      .dsrwith_inner(url, nskey, with_addr, subhandler_id, hand_mem)
      .await
    {
      Ok(hm) => hm,
      Err(_) => {
        let mut err_hm = HandlerMemory::new(None, 1).expect("what");
        err_hm
          .write_fractal(
            CLOSURE_ARG_MEM_START,
            &HandlerMemory::str_to_fractal("ERROR TODO"),
          )
          .expect("what");
        err_hm
      }
    }
  }

  async fn dsrwith_inner(
    self: &ControlPort,
    url: String,
    nskey: &str,
    with_addr: i64,
    subhandler_id: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> DaemonResult<Arc<HandlerMemory>> {
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req = req.header("nskey", nskey);
    let req = req.header("subhandler_id", format!("{}", subhandler_id));
    let mut hand_mem = HandlerMemory::fork(hand_mem.clone())?; // TODO: We need two of them!?
    hand_mem.register_out(with_addr, 1, CLOSURE_ARG_MEM_START)?;
    let mut out_hm = HandlerMemory::new(None, 2)?;
    HandlerMemory::transfer(
      &hand_mem,
      CLOSURE_ARG_MEM_START,
      &mut out_hm,
      CLOSURE_ARG_MEM_START + 2,
    )?;
    let mut out = vec![];
    out_hm.to_pb().write_to_vec(&mut out).unwrap();
    let req_obj = req.body(Body::from(out))?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
    Ok(HandlerMemory::from_pb(&pb)?)
  }

  pub async fn dsmwith(
    self: &ControlPort,
    nskey: &str,
    with_addr: i64,
    subhandler_id: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> Arc<HandlerMemory> {
    let vm = self.get_vm_for_key(nskey);
    // TODO: Use private ip if possible
    let url = format!("https://{}:4142/datastore/dsmwith", vm.public_ip_addr);
    match self
      .dsmwith_inner(url, nskey, with_addr, subhandler_id, hand_mem)
      .await
    {
      Ok(hm) => hm,
      Err(_) => {
        let mut err_hm = HandlerMemory::new(None, 1).expect("what");
        err_hm
          .write_fractal(
            CLOSURE_ARG_MEM_START,
            &HandlerMemory::str_to_fractal("ERROR TODO"),
          )
          .expect("what");
        err_hm
      }
    }
  }

  async fn dsmwith_inner(
    self: &ControlPort,
    url: String,
    nskey: &str,
    with_addr: i64,
    subhandler_id: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> DaemonResult<Arc<HandlerMemory>> {
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req = req.header("nskey", nskey);
    let req = req.header("subhandler_id", format!("{}", subhandler_id));
    let mut hand_mem = HandlerMemory::fork(hand_mem.clone())?; // TODO: We need two of them!?
    hand_mem.register_out(with_addr, 1, CLOSURE_ARG_MEM_START)?;
    let mut out_hm = HandlerMemory::new(None, 2)?;
    HandlerMemory::transfer(
      &hand_mem,
      CLOSURE_ARG_MEM_START,
      &mut out_hm,
      CLOSURE_ARG_MEM_START + 2,
    )?;
    let mut out = vec![];
    out_hm.to_pb().write_to_vec(&mut out).unwrap();
    let req_obj = req.body(Body::from(out))?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
    Ok(HandlerMemory::from_pb(&pb)?)
  }

  pub fn dsmonly(
    self: &ControlPort,
    nskey: &str,
    subhandler_id: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) {
    let vm = self.get_vm_for_key(nskey);
    // TODO: Use private ip if possible
    let url = format!("https://{}:4142/datastore/dsmonly", vm.public_ip_addr);
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req = req.header("nskey", nskey);
    let req = req.header("subhandler_id", format!("{}", subhandler_id));
    let orphan_hm = HandlerMemory::fork(hand_mem.clone()).expect("what"); // TODO: This clone is a terrible idea
    let orphan_hm = orphan_hm.drop_parent().expect("what");
    let mut out = vec![];
    orphan_hm.to_pb().write_to_vec(&mut out).unwrap();
    let req_obj = req.body(Body::from(out)).expect("what");
    let client = self.client.clone();
    task::spawn(async move {
      client.request(req_obj).await;
    });
  }

  pub fn dswonly(
    self: &ControlPort,
    nskey: &str,
    with_addr: i64,
    subhandler_id: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) {
    let vm = self.get_vm_for_key(nskey);
    // TODO: Use private ip if possible
    let url = format!("https://{}:4142/datastore/dsmonly", vm.public_ip_addr);
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req = req.header("nskey", nskey);
    let req = req.header("subhandler_id", format!("{}", subhandler_id));
    let mut hand_mem = HandlerMemory::fork(hand_mem.clone()).expect("what"); // TODO: We need two of them!?
    hand_mem
      .register_out(with_addr, 1, CLOSURE_ARG_MEM_START)
      .expect("what");
    let mut out_hm = HandlerMemory::new(None, 2).expect("what");
    HandlerMemory::transfer(
      &hand_mem,
      CLOSURE_ARG_MEM_START,
      &mut out_hm,
      CLOSURE_ARG_MEM_START + 2,
    )
    .expect("what");
    let mut out = vec![];
    out_hm.to_pb().write_to_vec(&mut out).unwrap();
    let req_obj = req.body(Body::from(out)).expect("what");
    let client = self.client.clone();
    task::spawn(async move {
      client.request(req_obj).await;
    });
  }

  pub async fn dsrclos(
    self: &ControlPort,
    nskey: &str,
    subhandler_id: i64,
    ret_addr: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> Arc<HandlerMemory> {
    let vm = self.get_vm_for_key(nskey);
    // TODO: Use private ip if possible
    let url = format!("https://{}:4142/datastore/dsrclos", vm.public_ip_addr);
    match self
      .dsrclos_inner(url, nskey, subhandler_id, ret_addr, hand_mem)
      .await
    {
      Ok(hm) => hm,
      Err(_) => {
        let mut err_hm = HandlerMemory::new(None, 1).expect("what");
        err_hm
          .write_fractal(
            CLOSURE_ARG_MEM_START,
            &HandlerMemory::str_to_fractal("ERROR TODO"),
          )
          .expect("what");
        err_hm
      }
    }
  }

  async fn dsrclos_inner(
    self: &ControlPort,
    url: String,
    nskey: &str,
    subhandler_id: i64,
    ret_addr: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> DaemonResult<Arc<HandlerMemory>> {
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req = req.header("nskey", nskey);
    let req = req.header("subhandler_id", format!("{}", subhandler_id));
    let req = req.header("ret_addr", format!("{}", ret_addr));
    let mut out = vec![];
    hand_mem.to_pb().write_to_vec(&mut out).unwrap();
    let req_obj = req.body(Body::from(out))?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
    Ok(HandlerMemory::from_pb(&pb)?)
  }

  pub async fn dsmclos(
    self: &ControlPort,
    nskey: &str,
    subhandler_id: i64,
    ret_addr: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> Arc<HandlerMemory> {
    let vm = self.get_vm_for_key(nskey);
    // TODO: Use private ip if possible
    let url = format!("https://{}:4142/datastore/dsmclos", vm.public_ip_addr);
    match self
      .dsmclos_inner(url, nskey, subhandler_id, ret_addr, hand_mem)
      .await
    {
      Ok(hm) => hm,
      Err(_) => {
        let mut err_hm = HandlerMemory::new(None, 1).expect("what");
        err_hm
          .write_fractal(
            CLOSURE_ARG_MEM_START,
            &HandlerMemory::str_to_fractal("ERROR TODO"),
          )
          .expect("what");
        err_hm
      }
    }
  }

  async fn dsmclos_inner(
    self: &ControlPort,
    url: String,
    nskey: &str,
    subhandler_id: i64,
    ret_addr: i64,
    hand_mem: &Arc<HandlerMemory>,
  ) -> DaemonResult<Arc<HandlerMemory>> {
    let req = Request::builder().method("POST").uri(url);
    let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
    let req = req.header(cluster_secret.as_str(), "true");
    let req = req.header("nskey", nskey);
    let req = req.header("subhandler_id", format!("{}", subhandler_id));
    let req = req.header("ret_addr", format!("{}", ret_addr));
    let mut out = vec![];
    hand_mem.to_pb().write_to_vec(&mut out).unwrap();
    let req_obj = req.body(Body::from(out))?;
    let mut res = self.client.request(req_obj).await?;
    let bytes = hyper::body::to_bytes(res.body_mut()).await?;
    let pb = protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes)?;
    Ok(HandlerMemory::from_pb(&pb)?)
  }

  fn get_all_vms_by_ip(self: &ControlPort) -> Vec<String> {
    self
      .vms
      .iter()
      .map(|(k, v)| {
        if self.region_vms.contains_key(k) {
          v.private_ip_addr.clone()
        } else {
          v.public_ip_addr.clone()
        }
      })
      .collect()
  }

  async fn rebalance_data(self: &ControlPort) {
    // 1. Get lists of keys across all live nodes in the cluster
    // 2. For each list:
    // 2a. Determine if this node should have the key as the new primary owner, secondary owner, or
    //     not at all.
    // 2b. If the primary owner and this list is from the node that is the first secondary owner,
    //     push the key and node IP to a 'get' list.
    // 2c. If the primary owner and this list is from a node that should not own the key, push the
    //     key and the node IP to a 'del' list.
    // 2d. If a secondary owner and this list is from the node that is the primary owner, push the
    //     key and node IP to a 'get' list. Do not create a 'del' entry.
    // 3. Iterate through the 'get' list and grab the data for each key and store it.
    // 3a. On failure, expand the query to every node that should have the data and re-query again.
    // 3b. On failure again, query every node, period, for the data.
    // 3c. On final failure, abort. It may have been a race condition with a key that was
    //     explicitly deleted. TODO: Add log records to verify this?
    // 4. Iterate through the 'del' list and delete the data from the node that should no longer
    //    have it. Failure means that something else deleted it already, so it can be ignored.
    println!("Rebalancing Keys");
    let vms = self.get_all_vms_by_ip();
    let fake_vm = VMMetadata::fake_vm();
    let self_vm = self.self_vm.as_ref().unwrap_or(&fake_vm);
    let key_lists = join_all(vms.iter().map(|ip| self.dskeys(ip))).await;
    let mut get_list: Vec<(String, String)> = Vec::new(); // (Key, Source IP)
    let mut del_list: Vec<(String, String)> = Vec::new(); // (Key, Source IP)
    key_lists.iter().for_each(|(ip, key_list)| {
      let ip_str = ip.to_string();
      key_list.iter().for_each(|key| {
        let relevant_nodes = self.get_vms_for_key(key);
        if relevant_nodes.len() > 1 {
          // Don't try to do any of this if the cluster is just 1 node
          let self_in_list = relevant_nodes
            .iter()
            .any(|node| node.public_ip_addr == self_vm.public_ip_addr);
          if !self_in_list {
            return;
          }
          let primary_node = relevant_nodes[0];
          let first_secondary = relevant_nodes[1];
          let self_primary = primary_node.public_ip_addr == self_vm.public_ip_addr;
          if self_primary {
            let this_list_first_secondary =
              first_secondary.public_ip_addr == ip_str || first_secondary.private_ip_addr == ip_str;
            if this_list_first_secondary {
              get_list.push((key.to_string(), ip.to_string()));
            }
            let this_list_irrelevant = relevant_nodes
              .iter()
              .all(|node| node.public_ip_addr != ip_str && node.private_ip_addr != ip_str);
            if this_list_irrelevant {
              del_list.push((key.to_string(), ip.to_string()));
            }
          } else {
            let this_list_primary =
              primary_node.public_ip_addr == ip_str || primary_node.private_ip_addr == ip_str;
            if this_list_primary {
              get_list.push((key.to_string(), ip.to_string()));
            }
          }
        }
      });
    });
    for (key, ip) in get_list.iter() {
      // For our purposes, we don't want the query result-wrapped, so we have a special raw
      // endpoint to get the data from
      let url = format!("https://{}:4142/datastore/getr", ip);
      let req = Request::builder().method("POST").uri(url);
      let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
      let req = req.header(cluster_secret.as_str(), "true");
      let req_obj = req.body(Body::from(
        json!(DSGet {
          nskey: key.to_string()
        })
        .to_string(),
      ));
      let req_obj = match req_obj {
        Ok(req_obj) => req_obj,
        Err(e) => {
          error!(UnexpectedError, "Should be impossible {:?}", e).await;
          continue;
        }
      };
      let mut res = match self.client.request(req_obj).await {
        Ok(res) => res,
        Err(e) => {
          error!(UnexpectedError, "Could not talk to peer {:?}", e).await;
          continue;
        }
      };
      let bytes = match hyper::body::to_bytes(res.body_mut()).await {
        Ok(bytes) => bytes,
        Err(e) => {
          error!(UnexpectedError, "Could not read data from peer {:?}", e).await;
          continue;
        }
      };
      let pb = match protos::HandlerMemory::HandlerMemory::parse_from_bytes(&bytes) {
        Ok(pb) => pb,
        Err(e) => {
          error!(UnexpectedError, "Could not parse data from peer {:?}", e).await;
          continue;
        }
      };
      let hm = match HandlerMemory::from_pb(&pb) {
        Ok(hm) => hm,
        Err(e) => {
          error!(UnexpectedError, "This should be impossible {:?}", e).await;
          continue;
        }
      };
      DS.insert(key.to_string(), hm);
    }
    for (key, ip) in del_list.iter() {
      let url = format!("https://{}:4142/datastore/del", ip);
      let req = Request::builder().method("POST").uri(url);
      let cluster_secret = CLUSTER_SECRET.get().unwrap().clone().unwrap();
      let req = req.header(cluster_secret.as_str(), "true");
      let req_obj = req.body(Body::from(
        json!(DSGet {
          nskey: key.to_string()
        })
        .to_string(),
      ));
      let req_obj = match req_obj {
        Ok(req_obj) => req_obj,
        Err(e) => {
          error!(UnexpectedError, "Should be impossible {:?}", e).await;
          continue;
        }
      };
      match self.client.request(req_obj).await {
        Ok(res) => res,
        Err(e) => {
          error!(UnexpectedError, "Could not talk to peer {:?}", e).await;
          continue;
        }
      };
    }
  }
}
