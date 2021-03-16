use std::net::SocketAddr;
use std::sync::Arc;
use std::convert::Infallible;
use std::io::{self, Write, BufReader};
use std::pin::Pin;
use futures::task::{Context, Poll};
use futures::future::Future;

use hyper::service::{make_service_fn, service_fn};
use async_stream::stream;
use hyper_rustls::HttpsConnector;
use futures_util::stream::Stream;
use hyper::{client::{Client, HttpConnector, ResponseFuture}, server::Server, Body, Request, Response, StatusCode};
use once_cell::sync::Lazy;
use rustls::internal::pemfile;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::TlsAcceptor;
use tokio_rustls::server::TlsStream;

#[derive(Debug)]
pub struct HttpConfig {
  pub port: u16,
}

#[derive(Debug)]
pub struct HttpsConfig {
  pub port: u16,
  pub priv_key_b64: String,
  pub cert_b64: String,
}

#[derive(Debug)]
pub enum HttpType {
  HTTP(HttpConfig),
  HTTPS(HttpsConfig),
}

struct HyperAcceptor<'a> {
  acceptor: Pin<Box<dyn Stream<Item = Result<TlsStream<TcpStream>, io::Error>> + Send + 'a>>,
}

impl hyper::server::accept::Accept for HyperAcceptor<'_> {
  type Conn = TlsStream<TcpStream>;
  type Error = io::Error;

  fn poll_accept(
    mut self: Pin<&mut Self>,
    cx: &mut Context,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
    Pin::new(&mut self.acceptor).poll_next(cx)
  }
}

pub static HTTP_CLIENT: Lazy<Client<HttpsConnector<HttpConnector>>> =
  Lazy::new(|| Client::builder().build::<_, Body>(HttpsConnector::with_native_roots()));

async fn make_http_server<L: 'static, Fut: 'static>(
  http: &HttpConfig,
  listener: L,
) where
  L: FnMut(Request<Body>) -> Fut + Send + Copy + Sync,
  Fut: Future<Output = Result<Response<Body>, Infallible>> + Send,
{
  let port_num = http.port;
  let addr = SocketAddr::from(([0, 0, 0, 0], port_num));
  let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(listener)) });

  let bind = Server::try_bind(&addr);
  match bind {
    Ok(server) => {
      let server = server.serve(make_svc);
      tokio::spawn(async move { server.await });
      println!("HTTP server listening on port {}", port_num);
    },
    Err(ee) => eprintln!("HTTP server failed to listen on port {}: {}", port_num, ee),
  }
}

async fn make_https_server<L: 'static, Fut: 'static>(
  https: &HttpsConfig,
  listener: L,
) where
  L: FnMut(Request<Body>) -> Fut + Send + Copy + Sync,
  Fut: Future<Output = Result<Response<Body>, Infallible>> + Send,
{
  let port_num = https.port;
  let addr = SocketAddr::from(([0, 0, 0, 0], port_num));
  let tls_cfg = {
    let certs = pemfile::certs(
      &mut BufReader::new(base64::decode(https.cert_b64.as_str()).unwrap().as_slice())
    );
    let certs = certs.expect("Failed to load certificate");
    let key = {
      let keys = pemfile::pkcs8_private_keys(
        &mut BufReader::new(
          base64::decode(https.priv_key_b64.as_str()).unwrap().as_slice()
        )
      );
      let keys = keys.expect("Failed to load private key");
      if keys.len() != 1 {
        panic!("Expected a single private key");
      }
      keys[0].clone()
    };
    let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
    cfg.set_single_cert(certs, key).unwrap();
    cfg.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
    Arc::new(cfg)
  };
  let tcp = TcpListener::bind(&addr).await;
  let tcp = tcp.unwrap();
  let tls_acceptor = TlsAcceptor::from(tls_cfg);
  let incoming_tls_stream = stream! {
    loop {
      let accept = tcp.accept().await;
      if accept.is_err() { continue; }
      let (socket, _) = accept.unwrap();
      let strm = tls_acceptor.accept(socket).into_failable();
      let strm_val = strm.await;
      if strm_val.is_err() { continue; }
      yield Ok(strm_val.unwrap());
    }
  };
  let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(listener)) });
  let server = Server::builder(HyperAcceptor {
    acceptor: Box::pin(incoming_tls_stream),
  }).serve(make_svc);
  println!("HTTPS server listening on port {}", port_num);
  tokio::spawn(async move { server.await });
}

pub async fn make_server<L: 'static, Fut: 'static>(
  config: &HttpType,
  listener: L,
) where
  L: FnMut(Request<Body>) -> Fut + Send + Copy + Sync,
  Fut: Future<Output = Result<Response<Body>, Infallible>> + Send,
{
  match config {
    HttpType::HTTP(http) => {
      make_http_server(http, listener);
    },
    HttpType::HTTPS(https) => {
      make_https_server(https, listener);
    },
  };
}
