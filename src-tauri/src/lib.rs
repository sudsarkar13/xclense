use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
  pub service: String,
  pub status: String,
  pub version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StorageSummary {
  pub total_bytes: u64,
  pub used_bytes: u64,
  pub free_bytes: u64,
  pub used_percent: f64,
  pub scanned_at_epoch_ms: u128,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessInfo {
  pub pid: i32,
  pub name: String,
  pub cpu_percent: f32,
  pub memory_percent: f32,
  pub state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IssueReport {
  pub id: String,
  pub title: String,
  pub severity: String,
  pub confidence: f64,
  pub evidence: Vec<String>,
  pub recommendation: String,
  pub suggested_action: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisReport {
  pub generated_at_epoch_ms: u128,
  pub total_issues: usize,
  pub issues: Vec<IssueReport>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReportSnapshotMeta {
  pub snapshot_id: String,
  pub created_at_epoch_ms: u128,
  pub issue_count: usize,
  pub highest_severity: String,
  pub source_version: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ReportSnapshot {
  pub meta: ReportSnapshotMeta,
  pub report: AnalysisReport,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
  pub snapshot_id: String,
  pub format: String,
  pub exported_at_epoch_ms: u128,
  pub file_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessActionConfirmation {
  pub acknowledged_risk: bool,
  pub reason: String,
  pub typed_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ManageProcessActionRequest {
  pub pid: i32,
  pub action: String,
  pub process_name_hint: Option<String>,
  pub confirmation: Option<ProcessActionConfirmation>,
  pub source_context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActionResult {
  pub action: String,
  pub target_pid: i32,
  pub status: String,
  pub message: String,
  pub performed_at_epoch_ms: u128,
  pub audit_id: String,
  pub risk_level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActionAuditRecord {
  pub audit_id: String,
  pub action: String,
  pub pid: i32,
  pub process_name: String,
  pub decision: String,
  pub reason: String,
  pub risk_level: String,
  pub requested_at_epoch_ms: u128,
  pub completed_at_epoch_ms: Option<u128>,
  pub source_version: String,
  pub source_context: Option<String>,
}

mod commands {
  use super::*;

  const MAX_SNAPSHOT_COUNT: usize = 100;
  const MAX_AUDIT_RECORDS: usize = 500;

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

  fn ensure_directory(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path)
      .map_err(|error| format!("failed to create directory '{}': {}", path.display(), error))
  }

  fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let temp_path = path.with_extension("tmp");
    let serialized = serde_json::to_vec_pretty(value)
      .map_err(|error| format!("failed to serialize json: {}", error))?;

    {
      let mut temp_file = fs::File::create(&temp_path)
        .map_err(|error| format!("failed to create temp file '{}': {}", temp_path.display(), error))?;
      temp_file
        .write_all(&serialized)
        .map_err(|error| format!("failed to write temp file '{}': {}", temp_path.display(), error))?;
      temp_file
        .sync_all()
        .map_err(|error| format!("failed to sync temp file '{}': {}", temp_path.display(), error))?;
    }

    fs::rename(&temp_path, path)
      .map_err(|error| format!("failed to finalize file '{}': {}", path.display(), error))
  }

  fn snapshots_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data = app
      .path()
      .app_data_dir()
      .map_err(|error| format!("failed to resolve app data directory: {}", error))?;

    let path = app_data.join("report-snapshots");
    ensure_directory(&path)?;
    Ok(path)
  }

  fn exports_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data = app
      .path()
      .app_data_dir()
      .map_err(|error| format!("failed to resolve app data directory: {}", error))?;

    let path = app_data.join("report-exports");
    ensure_directory(&path)?;
    Ok(path)
  }

  fn snapshot_file_path(app: &tauri::AppHandle, snapshot_id: &str) -> Result<PathBuf, String> {
    let dir = snapshots_dir(app)?;
    Ok(dir.join(format!("{}.json", snapshot_id)))
  }

  fn action_audit_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let app_data = app
      .path()
      .app_data_dir()
      .map_err(|error| format!("failed to resolve app data directory: {}", error))?;

    let path = app_data.join("action-audit");
    ensure_directory(&path)?;
    Ok(path)
  }

  fn action_audit_file_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = action_audit_dir(app)?;
    Ok(dir.join("process-actions.json"))
  }

  fn read_snapshot(path: &Path) -> Result<ReportSnapshot, String> {
    let content = fs::read_to_string(path)
      .map_err(|error| format!("failed to read snapshot file '{}': {}", path.display(), error))?;

    serde_json::from_str::<ReportSnapshot>(&content).map_err(|error| {
      format!(
        "failed to parse snapshot json from '{}': {}",
        path.display(),
        error
      )
    })
  }

  fn severity_rank(severity: &str) -> i32 {
    match severity {
      "critical" => 3,
      "warning" => 2,
      "info" => 1,
      _ => 0,
    }
  }

  fn highest_severity(issues: &[IssueReport]) -> String {
    if issues.iter().any(|issue| issue.severity == "critical") {
      return "critical".to_string();
    }

    if issues.iter().any(|issue| issue.severity == "warning") {
      return "warning".to_string();
    }

    if issues.iter().any(|issue| issue.severity == "info") {
      return "info".to_string();
    }

    "none".to_string()
  }

  fn enforce_snapshot_retention(app: &tauri::AppHandle) -> Result<(), String> {
    let directory = snapshots_dir(app)?;
    let entries = fs::read_dir(&directory)
      .map_err(|error| format!("failed to read snapshots directory '{}': {}", directory.display(), error))?;

    let mut metas: Vec<ReportSnapshotMeta> = Vec::new();

    for entry_result in entries {
      let entry = match entry_result {
        Ok(value) => value,
        Err(_) => continue,
      };

      let path = entry.path();
      if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
        continue;
      }

      if let Ok(snapshot) = read_snapshot(&path) {
        metas.push(snapshot.meta);
      }
    }

    metas.sort_by(|a, b| b.created_at_epoch_ms.cmp(&a.created_at_epoch_ms));

    for stale_meta in metas.iter().skip(MAX_SNAPSHOT_COUNT) {
      let stale_path = directory.join(format!("{}.json", stale_meta.snapshot_id));
      if stale_path.exists() {
        let _ = fs::remove_file(stale_path);
      }
    }

    Ok(())
  }

  fn enforce_audit_retention(path: &Path) -> Result<(), String> {
    if !path.exists() {
      return Ok(());
    }

    let content = fs::read_to_string(path)
      .map_err(|error| format!("failed to read audit file '{}': {}", path.display(), error))?;

    if content.trim().is_empty() {
      return Ok(());
    }

    let mut records: Vec<ActionAuditRecord> = serde_json::from_str(&content)
      .map_err(|error| format!("failed to parse audit file '{}': {}", path.display(), error))?;

    if records.len() > MAX_AUDIT_RECORDS {
      let start = records.len().saturating_sub(MAX_AUDIT_RECORDS);
      records = records.split_off(start);
      write_json_atomic(path, &records)?;
    }

    Ok(())
  }

  fn append_action_audit(
    app: &tauri::AppHandle,
    record: &ActionAuditRecord,
  ) -> Result<(), String> {
    let path = action_audit_file_path(app)?;

    let mut records: Vec<ActionAuditRecord> = if path.exists() {
      let content = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read audit file '{}': {}", path.display(), error))?;

      if content.trim().is_empty() {
        Vec::new()
      } else {
        serde_json::from_str(&content)
          .map_err(|error| format!("failed to parse audit json '{}': {}", path.display(), error))?
      }
    } else {
      Vec::new()
    };

    records.push(record.clone());
    write_json_atomic(&path, &records)?;
    enforce_audit_retention(&path)?;

    Ok(())
  }

  fn list_action_audits(app: &tauri::AppHandle, limit: usize) -> Result<Vec<ActionAuditRecord>, String> {
    let path = action_audit_file_path(app)?;
    if !path.exists() {
      return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)
      .map_err(|error| format!("failed to read audit file '{}': {}", path.display(), error))?;

    if content.trim().is_empty() {
      return Ok(Vec::new());
    }

    let mut records: Vec<ActionAuditRecord> = serde_json::from_str(&content)
      .map_err(|error| format!("failed to parse audit file '{}': {}", path.display(), error))?;

    records.sort_by(|a, b| b.requested_at_epoch_ms.cmp(&a.requested_at_epoch_ms));

    Ok(records.into_iter().take(limit).collect())
  }

  fn is_protected_process_name(process_name: &str) -> bool {
    let lower = process_name.to_lowercase();
    let deny = [
      "kernel_task",
      "launchd",
      "windowserver",
      "sysmond",
      "runningboardd",
      "logd",
      "mds",
      "securityd",
    ];

    deny.iter().any(|item| lower == *item)
  }

  fn evaluate_risk_level(process: &ProcessInfo) -> String {
    if is_protected_process_name(&process.name) || process.pid <= 1 {
      return "critical".to_string();
    }

    if process.cpu_percent >= 80.0 || process.memory_percent >= 30.0 {
      return "high".to_string();
    }

    if process.cpu_percent >= 40.0 || process.memory_percent >= 15.0 {
      return "medium".to_string();
    }

    "low".to_string()
  }

  fn requires_confirmation(action: &str, risk_level: &str) -> bool {
    action == "force_kill" || risk_level == "high" || risk_level == "critical"
  }

  fn validate_confirmation(
    request: &ManageProcessActionRequest,
    risk_level: &str,
  ) -> Result<(), String> {
    if !requires_confirmation(&request.action, risk_level) {
      return Ok(());
    }

    let confirmation = request
      .confirmation
      .as_ref()
      .ok_or_else(|| "confirmation is required for this action".to_string())?;

    if !confirmation.acknowledged_risk {
      return Err("risk acknowledgement is required".to_string());
    }

    if confirmation.reason.trim().is_empty() {
      return Err("confirmation reason is required".to_string());
    }

    if request.action == "force_kill" {
      let expected = format!("KILL {}", request.pid);
      let typed = confirmation
        .typed_token
        .as_ref()
        .ok_or_else(|| "typed token is required for force kill".to_string())?;

      if typed.trim() != expected {
        return Err(format!("invalid typed token. expected '{}'", expected));
      }
    }

    Ok(())
  }

  fn execute_process_action(pid: i32, action: &str) -> Result<(), String> {
    let signal = if action == "force_kill" { "-9" } else { "-15" };

    let status = Command::new("kill")
      .args([signal, &pid.to_string()])
      .status()
      .map_err(|error| format!("failed to invoke kill command: {}", error))?;

    if !status.success() {
      return Err(format!(
        "process action failed for pid {} with status {:?}",
        pid,
        status.code()
      ));
    }

    Ok(())
  }

  fn build_plain_text_report(snapshot: &ReportSnapshot) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("Xclense Analysis Snapshot".to_string());
    lines.push(format!("Snapshot ID: {}", snapshot.meta.snapshot_id));
    lines.push(format!("Created At (epoch ms): {}", snapshot.meta.created_at_epoch_ms));
    lines.push(format!("Issue Count: {}", snapshot.meta.issue_count));
    lines.push(format!("Highest Severity: {}", snapshot.meta.highest_severity));
    lines.push(format!("Source Version: {}", snapshot.meta.source_version));
    lines.push(String::new());

    if snapshot.report.issues.is_empty() {
      lines.push("No issues detected in this snapshot.".to_string());
    } else {
      lines.push("Issues:".to_string());
      for issue in &snapshot.report.issues {
        lines.push(format!("- [{}] {} (confidence: {:.2})", issue.severity.to_uppercase(), issue.title, issue.confidence));
        lines.push(format!("  Recommendation: {}", issue.recommendation));
        lines.push(format!("  Suggested Action: {}", issue.suggested_action));

        if !issue.evidence.is_empty() {
          lines.push("  Evidence:".to_string());
          for item in &issue.evidence {
            lines.push(format!("    - {}", item));
          }
        }

        lines.push(String::new());
      }
    }

    lines.join("\n")
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

  #[tauri::command]
  pub fn analyze_issues() -> Result<AnalysisReport, String> {
    let storage = scan_storage()?;
    let processes = list_processes()?;
    let health = get_system_health()?;

    let mut issues: Vec<IssueReport> = Vec::new();

    if storage.used_percent >= 90.0 {
      issues.push(IssueReport {
        id: "storage-critical-001".to_string(),
        title: "Critical disk saturation".to_string(),
        severity: "critical".to_string(),
        confidence: 0.97,
        evidence: vec![
          format!("Root volume usage is {:.2}%", storage.used_percent),
          format!("Free space is {} bytes", storage.free_bytes),
        ],
        recommendation:
          "Immediately free disk space by removing large files or moving archives off-device."
            .to_string(),
        suggested_action: "run_storage_cleanup".to_string(),
      });
    } else if storage.used_percent >= 75.0 {
      issues.push(IssueReport {
        id: "storage-warning-001".to_string(),
        title: "High disk utilization".to_string(),
        severity: "warning".to_string(),
        confidence: 0.9,
        evidence: vec![format!("Root volume usage is {:.2}%", storage.used_percent)],
        recommendation:
          "Review Downloads, caches, and unused applications to recover space proactively."
            .to_string(),
        suggested_action: "review_storage_usage".to_string(),
      });
    }

    if health.memory_pressure_percent >= 90.0 {
      issues.push(IssueReport {
        id: "memory-critical-001".to_string(),
        title: "Critical memory pressure".to_string(),
        severity: "critical".to_string(),
        confidence: 0.95,
        evidence: vec![format!(
          "Estimated memory pressure is {:.2}%",
          health.memory_pressure_percent
        )],
        recommendation:
          "Close memory-heavy applications and restart long-running workloads to avoid instability."
            .to_string(),
        suggested_action: "reduce_memory_pressure".to_string(),
      });
    } else if health.memory_pressure_percent >= 80.0 {
      issues.push(IssueReport {
        id: "memory-warning-001".to_string(),
        title: "Elevated memory pressure".to_string(),
        severity: "warning".to_string(),
        confidence: 0.88,
        evidence: vec![format!(
          "Estimated memory pressure is {:.2}%",
          health.memory_pressure_percent
        )],
        recommendation:
          "Identify high-memory apps and close non-essential sessions before pressure worsens."
            .to_string(),
        suggested_action: "inspect_memory_consumers".to_string(),
      });
    }

    let heavy_processes: Vec<&ProcessInfo> = processes
      .iter()
      .filter(|process| process.cpu_percent >= 70.0 || process.memory_percent >= 25.0)
      .take(5)
      .collect();

    if !heavy_processes.is_empty() {
      let evidence = heavy_processes
        .iter()
        .map(|process| {
          format!(
            "pid={} name={} cpu={:.2}% mem={:.2}%",
            process.pid, process.name, process.cpu_percent, process.memory_percent
          )
        })
        .collect();

      let severity = if heavy_processes
        .iter()
        .any(|process| process.cpu_percent >= 90.0 || process.memory_percent >= 45.0)
      {
        "warning"
      } else {
        "info"
      };

      issues.push(IssueReport {
        id: "process-hotspots-001".to_string(),
        title: "Resource-heavy active processes".to_string(),
        severity: severity.to_string(),
        confidence: if severity == "warning" { 0.86 } else { 0.82 },
        evidence,
        recommendation:
          "Review listed processes and terminate only non-critical workloads with sustained high usage."
            .to_string(),
        suggested_action: "review_process_candidates".to_string(),
      });
    }

    Ok(AnalysisReport {
      generated_at_epoch_ms: now_epoch_ms(),
      total_issues: issues.len(),
      issues,
    })
  }

  #[tauri::command]
  pub fn create_report_snapshot(
    app: tauri::AppHandle,
    report: Option<AnalysisReport>,
  ) -> Result<ReportSnapshotMeta, String> {
    let report_data = match report {
      Some(value) => value,
      None => analyze_issues()?,
    };

    let created_at_epoch_ms = now_epoch_ms();
    let snapshot_id = format!("rs-{}-{}", created_at_epoch_ms, report_data.total_issues);

    let meta = ReportSnapshotMeta {
      snapshot_id: snapshot_id.clone(),
      created_at_epoch_ms,
      issue_count: report_data.total_issues,
      highest_severity: highest_severity(&report_data.issues),
      source_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let snapshot = ReportSnapshot {
      meta: meta.clone(),
      report: report_data,
    };

    let path = snapshot_file_path(&app, &snapshot_id)?;
    write_json_atomic(&path, &snapshot)?;
    enforce_snapshot_retention(&app)?;

    Ok(meta)
  }

  #[tauri::command]
  pub fn list_report_snapshots(
    app: tauri::AppHandle,
    limit: Option<usize>,
  ) -> Result<Vec<ReportSnapshotMeta>, String> {
    let directory = snapshots_dir(&app)?;
    let entries = fs::read_dir(&directory)
      .map_err(|error| format!("failed to read snapshots directory '{}': {}", directory.display(), error))?;

    let mut metas: Vec<ReportSnapshotMeta> = Vec::new();

    for entry_result in entries {
      let entry = match entry_result {
        Ok(value) => value,
        Err(_) => continue,
      };

      let path = entry.path();
      if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
        continue;
      }

      if let Ok(snapshot) = read_snapshot(&path) {
        metas.push(snapshot.meta);
      }
    }

    metas.sort_by(|a, b| {
      b.created_at_epoch_ms
        .cmp(&a.created_at_epoch_ms)
        .then_with(|| severity_rank(&b.highest_severity).cmp(&severity_rank(&a.highest_severity)))
    });

    let capped_limit = limit.unwrap_or(20).min(100);
    Ok(metas.into_iter().take(capped_limit).collect())
  }

  #[tauri::command]
  pub fn get_report_snapshot(
    app: tauri::AppHandle,
    snapshot_id: String,
  ) -> Result<ReportSnapshot, String> {
    let path = snapshot_file_path(&app, &snapshot_id)?;
    if !path.exists() {
      return Err(format!("snapshot '{}' not found", snapshot_id));
    }

    read_snapshot(&path)
  }

  #[tauri::command]
  pub fn export_report_snapshot(
    app: tauri::AppHandle,
    snapshot_id: String,
    format: String,
  ) -> Result<ExportResult, String> {
    let normalized = format.to_lowercase();
    if normalized != "json" && normalized != "txt" {
      return Err("unsupported format. Use 'json' or 'txt'.".to_string());
    }

    let snapshot = get_report_snapshot(app.clone(), snapshot_id.clone())?;
    let export_directory = exports_dir(&app)?;

    let extension = if normalized == "json" { "json" } else { "txt" };
    let export_path = export_directory.join(format!("{}-export.{}", snapshot_id, extension));

    if normalized == "json" {
      write_json_atomic(&export_path, &snapshot)?;
    } else {
      let text_report = build_plain_text_report(&snapshot);
      fs::write(&export_path, text_report).map_err(|error| {
        format!(
          "failed to write text export '{}': {}",
          export_path.display(),
          error
        )
      })?;
    }

    Ok(ExportResult {
      snapshot_id,
      format: normalized,
      exported_at_epoch_ms: now_epoch_ms(),
      file_path: export_path.display().to_string(),
    })
  }

  #[tauri::command]
  pub fn manage_process_action(
    app: tauri::AppHandle,
    request: ManageProcessActionRequest,
  ) -> Result<ActionResult, String> {
    let requested_at = now_epoch_ms();
    let audit_id = format!("audit-{}-{}", requested_at, request.pid);

    let process = list_processes()?
      .into_iter()
      .find(|item| item.pid == request.pid)
      .ok_or_else(|| format!("process '{}' not found", request.pid))?;

    let process_name = request
      .process_name_hint
      .clone()
      .unwrap_or_else(|| process.name.clone());

    let risk_level = evaluate_risk_level(&process);

    if request.action != "terminate" && request.action != "force_kill" {
      return Err("unsupported process action. Use 'terminate' or 'force_kill'.".to_string());
    }

    if is_protected_process_name(&process.name) || process.pid <= 1 {
      let record = ActionAuditRecord {
        audit_id: audit_id.clone(),
        action: request.action.clone(),
        pid: request.pid,
        process_name,
        decision: "blocked".to_string(),
        reason: "protected process cannot be controlled".to_string(),
        risk_level: "critical".to_string(),
        requested_at_epoch_ms: requested_at,
        completed_at_epoch_ms: Some(now_epoch_ms()),
        source_version: env!("CARGO_PKG_VERSION").to_string(),
        source_context: request.source_context.clone(),
      };

      append_action_audit(&app, &record)?;

      return Ok(ActionResult {
        action: request.action,
        target_pid: request.pid,
        status: "blocked".to_string(),
        message: "protected process cannot be controlled".to_string(),
        performed_at_epoch_ms: now_epoch_ms(),
        audit_id,
        risk_level: "critical".to_string(),
      });
    }

    if let Err(error) = validate_confirmation(&request, &risk_level) {
      let record = ActionAuditRecord {
        audit_id: audit_id.clone(),
        action: request.action.clone(),
        pid: request.pid,
        process_name,
        decision: "denied".to_string(),
        reason: error.clone(),
        risk_level: risk_level.clone(),
        requested_at_epoch_ms: requested_at,
        completed_at_epoch_ms: Some(now_epoch_ms()),
        source_version: env!("CARGO_PKG_VERSION").to_string(),
        source_context: request.source_context.clone(),
      };

      append_action_audit(&app, &record)?;

      return Ok(ActionResult {
        action: request.action,
        target_pid: request.pid,
        status: "denied".to_string(),
        message: error,
        performed_at_epoch_ms: now_epoch_ms(),
        audit_id,
        risk_level,
      });
    }

    let execution = execute_process_action(request.pid, &request.action);
    let (status, message, decision) = match execution {
      Ok(_) => (
        "executed".to_string(),
        format!("{} action completed for pid {}", request.action, request.pid),
        "executed".to_string(),
      ),
      Err(error) => (
        "failed".to_string(),
        format!("process action failed: {}", error),
        "failed".to_string(),
      ),
    };

    let record = ActionAuditRecord {
      audit_id: audit_id.clone(),
      action: request.action.clone(),
      pid: request.pid,
      process_name,
      decision,
      reason: message.clone(),
      risk_level: risk_level.clone(),
      requested_at_epoch_ms: requested_at,
      completed_at_epoch_ms: Some(now_epoch_ms()),
      source_version: env!("CARGO_PKG_VERSION").to_string(),
      source_context: request.source_context,
    };

    append_action_audit(&app, &record)?;

    Ok(ActionResult {
      action: request.action,
      target_pid: request.pid,
      status,
      message,
      performed_at_epoch_ms: now_epoch_ms(),
      audit_id,
      risk_level,
    })
  }

  #[tauri::command]
  pub fn list_process_action_audits(
    app: tauri::AppHandle,
    limit: Option<usize>,
  ) -> Result<Vec<ActionAuditRecord>, String> {
    let capped_limit = limit.unwrap_or(20).min(100);
    list_action_audits(&app, capped_limit)
  }
}

pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      commands::ping_backend,
      commands::scan_storage,
      commands::list_processes,
      commands::get_system_health,
      commands::analyze_issues,
      commands::create_report_snapshot,
      commands::list_report_snapshots,
      commands::get_report_snapshot,
      commands::export_report_snapshot,
      commands::manage_process_action,
      commands::list_process_action_audits
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
