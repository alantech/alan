use serde_json::json;

const AMPLITUDE_API_KEY: &str = "ae20dafe801eddecf308c6ce643e19d1";
const AMPLITUDE_URL: &str = "https://api.amplitude.com/2/httpapi";
// https://doc.rust-lang.org/cargo/reference/environment-variables.html
const ALAN_VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");
const NO_TELEMETRY: Option<&'static str> = option_env!("ALAN_TELEMETRY_OFF");
const OS: &str = std::env::consts::OS;

pub async fn log() {
  let no_telemetry = NO_TELEMETRY.unwrap_or("false") == "true";
  if no_telemetry { return; }
  let body = json!({
    "api_key": AMPLITUDE_API_KEY,
    "events": [
      {
        "user_id": "oneforallandallforone",
        "event_type": "avm-run",
        "app_version": ALAN_VERSION.unwrap(),
        "os_name": OS,
      }
    ]
  });
  let client = reqwest::Client::new();
  if client.post(AMPLITUDE_URL).json(&body).send().await.is_ok() {
    // Do nothing
  }
}