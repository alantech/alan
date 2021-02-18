#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(not(target_os = "linux"), path = "nonlinux.rs")]
pub mod daemon;
pub mod dns;
pub mod lrh;