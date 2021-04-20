#[repr(u64)]
pub enum ErrorType {
  InvalidPwd = 100,
  NoEnvFile = 101,
  GitChanges = 102,
  NoGit = 103,
  DeleteTmpAppTar = 104,
  NoDockerFile = 105,
  InvalidCredentialsFile = 108,
  InvalidAnycloudFile = 110,
  AuthFailed = 113,
  NoDnsVms = 114,
  NoStats = 115,
  NoClusterSecret = 116,
  NoDns = 117,
  NoPrivateIp = 118,
  NoDnsPrivateIp = 119,
  ScaleFailed = 120,
  PostFailed = 121,
  RunAgzFailed = 122,
  NoDaemonProps = 128,
  DaemonStartFailed = 129,
  CtrlPortStartFailed = 130,
}

#[macro_export]
macro_rules! error {
  ($errCode:expr, $($message:tt)+) => {async{
    eprintln!($($message)+);
    $crate::deploy::client_error($errCode, &format!($($message)+), "error").await;
  }};
  (metadata: $metadata:tt, $errCode:tt, $($message:tt)+) => {async{
    let value = json!($metadata);
    eprintln!($($message)+);
    $crate::deploy::client_error($errCode, &format!($($message)+), "error").await;
  }}
}

#[macro_export]
macro_rules! warn {
  ($errCode:expr, $($message:tt)+) => {async{
    eprintln!($($message)+);
    $crate::deploy::client_error($errCode, &format!($($message)+), "warn").await;
  }};
  (metadata: $metadata:tt, $errCode:tt, $($message:tt)+) => {async{
    let value = json!($metadata);
    eprintln!($($message)+);
    $crate::deploy::client_error($errCode, &format!($($message)+), "warn").await;
  }}
}
