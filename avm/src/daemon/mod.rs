#[cfg_attr(linux, path = "linux.rs")]
#[cfg_attr(not(linux), path = "rest.rs")]
pub mod daemon;
pub mod dns;
pub mod lrh;