use futures::task::{Context, Poll};
use std::io;
use std::pin::Pin;

use futures_util::stream::Stream;
use hyper::{
  client::{Client, HttpConnector},
  Body,
};
use hyper_rustls::HttpsConnector;
use once_cell::sync::Lazy;
use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

#[derive(Debug)]
pub struct HttpConfig {
  pub port: u16,
}

#[derive(Debug)]
pub struct HttpsConfig {
  pub port: u16,
  pub priv_key: String,
  pub cert: String,
}

#[derive(Debug)]
pub enum HttpType {
  HTTP(HttpConfig),
  HTTPS(HttpsConfig),
}

pub struct HyperAcceptor<'a> {
  pub acceptor: Pin<Box<dyn Stream<Item = Result<TlsStream<TcpStream>, io::Error>> + Send + 'a>>,
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

#[macro_export]
macro_rules! make_server {
  ($config:expr, $listener:expr) => {
    match $config {
      crate::vm::http::HttpType::HTTP(http) => {
        let port_num = http.port;
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port_num));
        let make_svc = hyper::service::make_service_fn(|_conn| async {
          Ok::<_, std::convert::Infallible>(hyper::service::service_fn($listener))
        });

        let bind = hyper::server::Server::try_bind(&addr);
        match bind {
          Ok(server) => {
            let server = server.serve(make_svc);
            tokio::spawn(async move { server.await });
            println!("HTTP server listening on port {}", port_num);
          }
          Err(ee) => eprintln!("HTTP server failed to listen on port {}: {}", port_num, ee),
        };
      }
      crate::vm::http::HttpType::HTTPS(https) => {
        let port_num = https.port;
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port_num));
        let tls_cfg = {
          let certs = rustls::internal::pemfile::certs(&mut std::io::BufReader::new(
            https.cert.as_str().as_bytes(),
          ));
          let certs = certs.expect("Failed to load certificate");
          let key = {
            let keys = rustls::internal::pemfile::pkcs8_private_keys(&mut std::io::BufReader::new(
              https.priv_key.as_str().as_bytes(),
            ));
            let keys = keys.expect("Failed to load private key");
            if keys.len() != 1 {
              panic!("Expected a single private key");
            }
            keys[0].clone()
          };
          let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
          cfg.set_single_cert(certs, key).unwrap();
          cfg.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
          std::sync::Arc::new(cfg)
        };
        let tcp = tokio::net::TcpListener::bind(&addr).await;
        let tcp = tcp.unwrap();
        let tls_acceptor = tokio_rustls::TlsAcceptor::from(tls_cfg);
        let incoming_tls_stream = async_stream::stream! {
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
        let make_svc = hyper::service::make_service_fn(|_conn| async {
          Ok::<_, std::convert::Infallible>(hyper::service::service_fn($listener))
        });
        let server = hyper::server::Server::builder(crate::vm::http::HyperAcceptor {
          acceptor: Box::pin(incoming_tls_stream),
        })
        .serve(make_svc);
        println!("HTTPS server listening on port {}", port_num);
        tokio::spawn(async move { server.await });
      }
    };
  };
}

#[macro_export]
macro_rules! make_tunnel {
  ($config:expr, $dest_port:expr) => {{
    match $config {
      crate::vm::http::HttpType::HTTP(http) => {
        let port_num = http.port;
        eprintln!("Trying to listen on {}", http.port);
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port_num));
        let bind = tokio::net::TcpListener::bind(addr).await;
        match bind {
          Ok(server) => {
            tokio::spawn(async move {
              loop {
                let src_socket = server.accept().await;
                match src_socket {
                  Ok((mut src_stream, _src_addr)) => {
                    // Do we need the source address for anything?
                    tokio::spawn(async move {
                      let dest_socket =
                        tokio::net::TcpStream::connect(format!("127.0.0.1:{}", $dest_port)).await;
                      match dest_socket {
                        Ok(mut dest_stream) => {
                          let res = tokio::io::copy_bidirectional(
                            &mut src_stream,
                            &mut dest_stream
                          ).await;
                          match res {
                            Ok(_) => { /* Do nothing */ },
                            Err(_) => { /* Also do nothing? */ },
                          }
                        }
                        Err(ee) => eprintln!(
                          "Tunnel failed to connect to downstream on port {}: {}",
                          $dest_port, ee
                        ),
                      };
                    });
                  }
                  Err(ee) => eprintln!("Tunnel failed to open the socket? {}", ee),
                };
              }
            });
            true
          }
          Err(ee) => {
            eprintln!("TCP tunnel failed to listen on port {}: {}", port_num, ee);
            false
          }
        }
      }
      crate::vm::http::HttpType::HTTPS(https) => {
        use futures_util::StreamExt;
        let port_num = https.port;
        eprintln!("Trying to listen on {}", https.port);
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port_num));
        let tls_cfg = {
          let certs = rustls::internal::pemfile::certs(&mut std::io::BufReader::new(
            https.cert.as_str().as_bytes(),
          ));
          let certs = certs.expect("Failed to load certificate");
          let key = {
            let keys = rustls::internal::pemfile::pkcs8_private_keys(&mut std::io::BufReader::new(
              https.priv_key.as_str().as_bytes(),
            ));
            let keys = keys.expect("Failed to load private key");
            if keys.len() != 1 {
              panic!("Expected a single private key");
            }
            keys[0].clone()
          };
          let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
          cfg.set_single_cert(certs, key).unwrap();
          cfg.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
          std::sync::Arc::new(cfg)
        };
        let tcp = tokio::net::TcpListener::bind(&addr).await;
        match tcp {
          Ok(tcp) => {
            tokio::spawn(async move {
              let tls_acceptor = tokio_rustls::TlsAcceptor::from(tls_cfg);
              let incoming_tls_stream = async_stream::stream! {
                loop {
                  let accept = tcp.accept().await;
                  if accept.is_err() { continue; }
                  let (socket, _) = accept.unwrap();
                  let strm = tls_acceptor.accept(socket).into_failable();
                  let strm_val = strm.await;
                  if strm_val.is_err() { continue; }
                  yield Ok(strm_val.unwrap());
                  if 1 == 0 { yield Err("never"); } // Make Rust type inference shut up :(
                }
              };
              futures_util::pin_mut!(incoming_tls_stream);
              while let Some(src_socket) = incoming_tls_stream.next().await {
                tokio::spawn(async move {
                  eprintln!("Hi there!");
                  eprintln!("{:?}", src_socket);
                  //use tokio::io::{AsyncRead, AsyncReadExt};
                  match src_socket {
                    Ok(mut src_stream) => {
                      /*let mut buf = [0; 4096];
                      let amount = src_stream.read(&mut buf).await;
                      eprintln!("amount {:?}", amount);
                      drop(amount);
                      eprintln!("buf {:?}", buf);*/
                      // Do we need the source address for anything?
                      // Maybe load balancing?
                      let dest_socket =
                        tokio::net::TcpStream::connect(format!("127.0.0.1:{}", $dest_port)).await;
                      match dest_socket {
                        Ok(mut dest_stream) => {
                          eprintln!("Hi again");
                          let res = tokio::io::copy_bidirectional(
                            &mut src_stream,
                            &mut dest_stream
                          ).await;
                          match res {
                            Ok(_) => { /* Do nothing */ },
                            Err(e) => { panic!("Wut {:?}", e); /* Also do nothing? */ },
                          }
                        }
                        Err(ee) => eprintln!(
                          "Tunnel failed to connect to downstream on port {}: {}",
                          $dest_port, ee
                        ),
                      };
                    }
                    Err(ee) => eprintln!("Tunnel failed to open the socket? {}", ee),
                  };
                });
              }
            });
            true
          },
          Err(ee) => {
            eprintln!("TLS tunnel failed to listen on port {}: {}", port_num, ee);
            false
          }
        }
      }
    }
  }};
}
