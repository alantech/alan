use hyper::{client::Client, Body, Request};
use hyper_rustls::HttpsConnectorBuilder;

use serde_json::json;

const AMPLITUDE_API_KEY: &str = "ae20dafe801eddecf308c6ce643e19d1";
const AMPLITUDE_URL: &str = "https://api.amplitude.com/2/httpapi";
// https://doc.rust-lang.org/cargo/reference/environment-variables.html
const ALAN_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const NO_TELEMETRY: Option<&'static str> = option_env!("ALAN_TELEMETRY_OFF");
const OS: &str = std::env::consts::OS;

pub async fn log(event: &str) {
  let no_telemetry = NO_TELEMETRY.unwrap_or("false") == "true";
  if no_telemetry {
    return;
  }
  let body = json!({
    "api_key": AMPLITUDE_API_KEY,
    "events": [
      {
        "user_id": "alancli",
        "event_type": event,
        "app_version": ALAN_VERSION.unwrap(),
        "os_name": OS,
      }
    ]
  });
  let client =
    Client::builder().build::<_, Body>(HttpsConnectorBuilder::new().with_native_roots().https_or_http().enable_all_versions().build());
  if client
    .request(
      Request::post(AMPLITUDE_URL)
        .body(body.to_string().into())
        .unwrap(),
    )
    .await
    .is_ok()
  {
    // do nothing
  }
}
