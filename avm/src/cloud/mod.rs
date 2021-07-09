use once_cell::sync::OnceCell;

pub static CLUSTER_ID: OnceCell<String> = OnceCell::new();
// Macros need to be defined before used
#[macro_use]
pub mod logger;
pub mod common;
pub mod deploy;
pub mod http;
pub mod oauth;
