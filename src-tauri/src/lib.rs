use serde::Serialize;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
  pub service: String,
  pub status: String,
  pub version: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageSummary {
  pub total_bytes: u64,
  pub used_bytes: u64,
  pub free_bytes: u64,
  pub used_percent: f64,
  pub scanned_at_epoch_ms: u128,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
  pub pid: i32,
  pub name: String,
  pub cpu_percent: f32,
  pub memory_percent: f32,
  pub state: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemHealth {
  pub memory_total_bytes: u64,
  pub memory_free_bytes: u64,
  pub memory_used_bytes: u64,
  pub memory_pressure_percent: f64,
  pub load_average_1m: f64,
  pub load_average_5m: f64,
  pub load_average_15m: f64,
  pub scanned_at_epoch_ms: u128,
}

mod commands {
  use super::*;

  fn now_epoch_ms() -> u128 {
    SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .map(|duration| duration.as_millis())
      .unwrap_or(0)
  }

  fn run(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
      .args(args)
      .output()
      .map_err(|error| format!("failed to run '{}': {}", command, error))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(format!(
        "command '{}' failed with status {:?}: {}",
        command,
        output.status.code(),
        stderr.trim()
      ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
  }

  fn parse_u64(value: &str) -> Result<u64, String> {
    value
      .replace(',', "")
      .trim()
      .parse::<u64>()
      .map_err(|error| format!("failed to parse u64 '{}': {}", value, error))
  }

  fn parse_f64(value: &str) -> Result<f64, String> {
    value
      .trim()
      .parse::<f64>()
      .map_err(|error| format!("failed to parse f64 '{}': {}", value, error))
  }

  #[tauri::command]
  pub fn ping_backend() -> PingResponse {
    PingResponse {
      service: "xclense-core".to_string(),
      status: "ok".to_string(),
      version: env!("CARGO_PKG_VERSION").to_string(),
    }
  }

  #[tauri::command]
  pub fn scan_storage() -> Result<StorageSummary, String> {
    let df_output = run("df", &["-k", "/"])?;
    let mut lines = df_output.lines();
    let _ = lines.next();
    let row = lines
      .next()
      .ok_or_else(|| "df output did not include filesystem row".to_string())?;

    let fields: Vec<&str> = row.split_whitespace().collect();
    if fields.len() < 5 {
      return Err("unexpected df output format".to_string());
    }

    let total_kb = parse_u64(fields[1])?;
    let used_kb = parse_u64(fields[2])?;
    let free_kb = parse_u64(fields[3])?;
    let used_percent_text = fields[4].trim_end_matches('%');
    let used_percent = parse_f64(used_percent_text)?;

    Ok(StorageSummary {
      total_bytes: total_kb.saturating_mul(1024),
      used_bytes: used_kb.saturating_mul(1024),
      free_bytes: free_kb.saturating_mul(1024),
      used_percent,
      scanned_at_epoch_ms: now_epoch_ms(),
    })
  }

  #[tauri::command]
  pub fn list_processes() -> Result<Vec<ProcessInfo>, String> {
    let ps_output = run("ps", &["-Ao", "pid=,comm=,%cpu=,%mem=,state=", "-r"])?;

    let mut processes: Vec<ProcessInfo> = Vec::new();

    for line in ps_output.lines().take(100) {
      let columns: Vec<&str> = line.split_whitespace().collect();
      if columns.len() < 5 {
        continue;
      }

      let pid = match columns[0].parse::<i32>() {
        Ok(value) => value,
        Err(_) => continue,
      };

      let name = columns[1].to_string();
      let cpu_percent = columns[2].parse::<f32>().unwrap_or(0.0);
      let memory_percent = columns[3].parse::<f32>().unwrap_or(0.0);
      let state = columns[4].to_string();

      processes.push(ProcessInfo {
        pid,
        name,
        cpu_percent,
        memory_percent,
        state,
      });
    }

    Ok(processes)
  }

  #[tauri::command]
  pub fn get_system_health() -> Result<SystemHealth, String> {
    let memsize_output = run("sysctl", &["-n", "hw.memsize"])?;
    let memory_total_bytes = parse_u64(&memsize_output)?;

    let vm_stat_output = run("vm_stat", &[])?;
    let page_size = vm_stat_output
      .lines()
      .next()
      .and_then(|header| {
        header
          .split("page size of ")
          .nth(1)
          .and_then(|rest| rest.split(" bytes").next())
      })
      .ok_or_else(|| "unable to determine vm_stat page size".to_string())
      .and_then(parse_u64)?;

    let mut free_pages = 0_u64;
    for line in vm_stat_output.lines() {
      if line.starts_with("Pages free:") {
        let value = line
          .split(':')
          .nth(1)
          .unwrap_or("0")
          .replace('.', "")
          .replace(',', "");
        free_pages = parse_u64(&value)?;
      }
    }

    let memory_free_bytes = free_pages.saturating_mul(page_size);
    let memory_used_bytes = memory_total_bytes.saturating_sub(memory_free_bytes);
    let memory_pressure_percent = if memory_total_bytes == 0 {
      0.0
    } else {
      (memory_used_bytes as f64 / memory_total_bytes as f64) * 100.0
    };

    let uptime_output = run("uptime", &[])?;
    let load_segment = uptime_output
      .split("load averages:")
      .nth(1)
      .ok_or_else(|| "unable to parse uptime load averages".to_string())?
      .trim();

    let load_values: Vec<&str> = load_segment
      .split_whitespace()
      .map(|token| token.trim_end_matches(','))
      .collect();

    if load_values.len() < 3 {
      return Err("uptime did not provide three load averages".to_string());
    }

    let load_average_1m = parse_f64(load_values[0])?;
    let load_average_5m = parse_f64(load_values[1])?;
    let load_average_15m = parse_f64(load_values[2])?;

    Ok(SystemHealth {
      memory_total_bytes,
      memory_free_bytes,
      memory_used_bytes,
      memory_pressure_percent,
      load_average_1m,
      load_average_5m,
      load_average_15m,
      scanned_at_epoch_ms: now_epoch_ms(),
    })
  }
}

pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      commands::ping_backend,
      commands::scan_storage,
      commands::list_processes,
      commands::get_system_health
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
