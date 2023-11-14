use hyper::{
  client::{Client, HttpConnector},
  Body,
};
use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};
use once_cell::sync::Lazy;

pub static CLIENT: Lazy<Client<HttpsConnector<HttpConnector>>> =
  Lazy::new(|| Client::builder().build::<_, Body>(HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_all_versions().build()));
