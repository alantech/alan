use tokio::net::TcpStream;
use tokio_rustls::server::TlsStream;

#[derive(Debug)]
pub struct TcpConfig {
  pub port: u16,
}

#[derive(Debug)]
pub struct TlsConfig {
  pub port: u16,
  pub priv_key_b64: String,
  pub cert_b64: String,
}

#[derive(Debug)]
pub enum TcpType {
  TCP(TcpConfig),
  TLS(TlsConfig),
}

#[macro_export]
macro_rules! make_tcp_server {
  ($config:expr, $listener:expr) => {
    match $config {
      crate::vm::tcp::TcpType::TCP(tcp) => {
        let port_num = tcp.port;
        let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port_num));
        let tcp = tokio::net::TcpListener::bind(&addr).await;
        let tcp = tcp.unwrap();
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
            ::base64::decode(https.cert_b64.as_str())
              .unwrap()
              .as_slice(),
          ));
          let certs = certs.expect("Failed to load certificate");
          let key = {
            let keys = rustls::internal::pemfile::pkcs8_private_keys(&mut std::io::BufReader::new(
              ::base64::decode(https.priv_key_b64.as_str())
                .unwrap()
                .as_slice(),
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
