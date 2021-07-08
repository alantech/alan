use hyper::{
  client::{Client, HttpConnector},
  Body,
};
use hyper_rustls::HttpsConnector;
use once_cell::sync::Lazy;

pub static CLIENT: Lazy<Client<HttpsConnector<HttpConnector>>> =
  Lazy::new(|| Client::builder().build::<_, Body>(HttpsConnector::with_native_roots()));
