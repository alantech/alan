use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::GzDecoder;
use tokio::runtime;

use crate::vm::run::exec;

pub fn start(agz_file: &str, app_id: &str, deploy_key: &str) {

}