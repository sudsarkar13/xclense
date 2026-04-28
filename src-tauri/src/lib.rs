use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
  pub service: String,
  pub status: String,
  pub version: String,
}

mod commands {
  use super::PingResponse;

  #[tauri::command]
  pub fn ping_backend() -> PingResponse {
    PingResponse {
      service: "xclense-core".to_string(),
      status: "ok".to_string(),
      version: env!("CARGO_PKG_VERSION").to_string(),
    }
  }
}

pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![commands::ping_backend])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
