use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager};

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
pub struct PhysicalDisk {
  pub device: String,
  pub kind: String,
  pub size_bytes: u64,
  pub removable: bool,
  pub internal: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct VolumeInfo {
  pub mount_point: String,
  pub filesystem: String,
  pub total_bytes: u64,
  pub used_bytes: u64,
  pub free_bytes: u64,
  pub used_percent: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StorageDetail {
  pub scanned_at_epoch_ms: u128,
  pub mac_model: String,
  pub architecture: String,
  pub macos_version: String,
  pub physical_disks: Vec<PhysicalDisk>,
  pub volumes: Vec<VolumeInfo>,
  pub summary: StorageSummary,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StorageCategory {
  pub id: String,
  pub label: String,
  pub description: String,
  pub color: String,
  pub path_prefixes: Vec<String>,
  pub risk_level: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StorageScanItem {
  pub id: String,
  pub category_id: String,
  pub path: String,
  pub size_bytes: u64,
  pub modified_epoch_ms: u128,
  pub last_accessed_epoch_ms: u128,
  pub risk_level: String,
  pub recommendation: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StorageScanResult {
  pub started_at_epoch_ms: u128,
  pub completed_at_epoch_ms: u128,
  pub scanned_paths: u32,
  pub items: Vec<StorageScanItem>,
  pub categories: Vec<StorageCategory>,
  pub total_recoverable_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CleanupRequest {
  pub item_ids: Vec<String>,
  pub acknowledged_risk: bool,
  pub reason: String,
  pub typed_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CleanupItemResult {
  pub item_id: String,
  pub path: String,
  pub status: String,
  pub message: String,
  pub reclaimed_bytes: u64,
  pub performed_at_epoch_ms: u128,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
  pub requested_item_ids: Vec<String>,
  pub results: Vec<CleanupItemResult>,
  pub total_reclaimed_bytes: u64,
  pub all_succeeded: bool,
  pub performed_at_epoch_ms: u128,
  pub audit_id: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CleanupProgressEvent {
  pub audit_id: String,
  pub phase: String,
  pub current: u32,
  pub total: u32,
  pub item_id: Option<String>,
  pub path: Option<String>,
  pub status: Option<String>,
  pub message: String,
  pub reclaimed_bytes: u64,
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
  pub categories: Vec<IssueCategory>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IssueCategory {
  pub id: String,
  pub label: String,
  pub severity: String,
  pub count: usize,
  pub first_issue_id: String,
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
  pub decision_code: String,
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
  pub decision_code: String,
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

  fn categorise_issue(issue: &IssueReport) -> (&'static str, &'static str) {
    let haystack = format!("{} {}", issue.id.to_lowercase(), issue.title.to_lowercase());

    if haystack.contains("memory")
      || haystack.contains("ram")
      || haystack.contains("swap")
      || haystack.contains("pressure")
    {
      return ("memory_pressure", "Memory pressure");
    }

    if haystack.contains("storage")
      || haystack.contains("disk")
      || haystack.contains("apfs snapshot")
    {
      return ("storage_full", "Storage full");
    }

    if haystack.contains("cpu")
      || haystack.contains("load average")
      || haystack.contains("process")
      || haystack.contains("zombie")
      || haystack.contains("orphan")
    {
      return ("process_load", "Heavy processes");
    }

    ("other", "Other findings")
  }

  fn build_categories(issues: &[IssueReport]) -> Vec<IssueCategory> {
    let mut grouped: std::collections::BTreeMap<
      &'static str,
      (String, String, String, usize, String),
    > = std::collections::BTreeMap::new();

    for issue in issues {
      let (cat_id, label) = categorise_issue(issue);
      let entry = grouped.entry(cat_id).or_insert_with(|| {
        (
          cat_id.to_string(),
          label.to_string(),
          "info".to_string(),
          0usize,
          String::new(),
        )
      });

      entry.3 += 1;
      if entry.4.is_empty() {
        entry.4 = issue.id.clone();
      }

      if severity_rank(&issue.severity) > severity_rank(&entry.2) {
        entry.2 = issue.severity.clone();
      }
    }

    grouped
      .into_iter()
      .map(|(_cat_id, (id, label, severity, count, first_issue_id))| IssueCategory {
        id,
        label,
        severity,
        count,
        first_issue_id,
      })
      .collect()
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
    let total_bytes = total_kb.saturating_mul(1024);
    let used_bytes = used_kb.saturating_mul(1024);
    let free_bytes = free_kb.saturating_mul(1024);
    let used_percent = if total_bytes == 0 {
      0.0
    } else {
      (used_bytes as f64 / total_bytes as f64) * 100.0
    };

    Ok(StorageSummary {
      total_bytes,
      used_bytes,
      free_bytes,
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
      categories: build_categories(&issues),
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
        decision_code: "POLICY_PROTECTED_PROCESS".to_string(),
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
        decision_code: "POLICY_PROTECTED_PROCESS".to_string(),
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
        decision_code: "CONFIRMATION_REQUIRED_OR_INVALID".to_string(),
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
        decision_code: "CONFIRMATION_REQUIRED_OR_INVALID".to_string(),
        performed_at_epoch_ms: now_epoch_ms(),
        audit_id,
        risk_level,
      });
    }

    let execution = execute_process_action(request.pid, &request.action);
    let (status, message, decision, decision_code) = match execution {
      Ok(_) => (
        "executed".to_string(),
        format!("{} action completed for pid {}", request.action, request.pid),
        "executed".to_string(),
        "ACTION_EXECUTED".to_string(),
      ),
      Err(error) => (
        "failed".to_string(),
        format!("process action failed: {}", error),
        "failed".to_string(),
        "ACTION_EXECUTION_FAILED".to_string(),
      ),
    };

    let record = ActionAuditRecord {
      audit_id: audit_id.clone(),
      action: request.action.clone(),
      pid: request.pid,
      process_name,
      decision,
      decision_code: decision_code.clone(),
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
      decision_code,
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

  #[derive(Debug, Serialize, Deserialize, Clone)]
  #[serde(rename_all = "camelCase")]
  pub struct RemediationStep {
    pub id: String,
    pub title: String,
    pub description: String,
    pub risk_level: String,
    pub auto_runnable: bool,
    pub guidance: Vec<String>,
  }

  #[derive(Debug, Serialize, Deserialize, Clone)]
  #[serde(rename_all = "camelCase")]
  pub struct RemediationPlan {
    pub generated_at_epoch_ms: u128,
    pub issue_count: usize,
    pub steps: Vec<RemediationStep>,
    pub auto_safe_steps: Vec<String>,
  }

  #[derive(Debug, Serialize, Deserialize, Clone)]
  #[serde(rename_all = "camelCase")]
  pub struct RemediationStepResult {
    pub step_id: String,
    pub status: String,
    pub message: String,
    pub performed_at_epoch_ms: u128,
  }

  #[derive(Debug, Serialize, Deserialize, Clone)]
  #[serde(rename_all = "camelCase")]
  pub struct RemediationExecution {
    pub requested_step_ids: Vec<String>,
    pub results: Vec<RemediationStepResult>,
    pub all_succeeded: bool,
  }

  fn build_safe_remediation_plan(report: &AnalysisReport) -> RemediationPlan {
    let mut steps: Vec<RemediationStep> = Vec::new();

    let has_memory_pressure = report.issues.iter().any(|issue| {
      issue.id.contains("memory")
        || issue.title.to_lowercase().contains("memory")
        || issue.title.to_lowercase().contains("pressure")
    });

    let has_storage_pressure = report.issues.iter().any(|issue| {
      issue.id.contains("storage")
        || issue.title.to_lowercase().contains("disk")
        || issue.title.to_lowercase().contains("storage")
    });

    let has_process_pressure = report.issues.iter().any(|issue| {
      issue.id.contains("process")
        || issue.id.contains("cpu")
        || issue.title.to_lowercase().contains("process")
        || issue.title.to_lowercase().contains("resource-heavy")
    });

    if has_memory_pressure {
      steps.push(RemediationStep {
        id: "free_inactive_memory".to_string(),
        title: "Free inactive memory".to_string(),
        description: "Ask macOS to release file-backed inactive memory pages. This is the standard, non-destructive way to recover RAM without killing apps.".to_string(),
        risk_level: "low".to_string(),
        auto_runnable: true,
        guidance: vec![
          "macOS will return inactive memory to the free pool automatically.".to_string(),
          "No user data or running apps are affected.".to_string(),
          "Expect a brief pause (under a second) while the kernel reclaims pages.".to_string(),
        ],
      });

      steps.push(RemediationStep {
        id: "close_heavy_tabs".to_string(),
        title: "Close heavy browser tabs and apps".to_string(),
        description: "Browser tabs and chat apps are the most common memory hogs. Closing the heaviest few usually recovers several GB.".to_string(),
        risk_level: "low".to_string(),
        auto_runnable: false,
        guidance: vec![
          "Open Activity Monitor → Memory tab and sort by Memory.".to_string(),
          "Quit the top 1–3 non-essential apps or close heavy browser tabs.".to_string(),
          "Re-check the health score after each one to see the effect.".to_string(),
        ],
      });

      steps.push(RemediationStep {
        id: "restart_browser".to_string(),
        title: "Restart your browser".to_string(),
        description: "Browsers leak memory over time. A clean restart typically reclaims 1–3 GB without losing bookmarks or logins.".to_string(),
        risk_level: "low".to_string(),
        auto_runnable: false,
        guidance: vec![
          "Save any unsaved work in browser tabs.".to_string(),
          "Quit and relaunch the browser — logins are restored automatically.".to_string(),
        ],
      });
    }

    if has_storage_pressure {
      steps.push(RemediationStep {
        id: "review_storage_usage".to_string(),
        title: "Review storage usage".to_string(),
        description: "Open the macOS Storage settings panel to identify large apps, caches, and documents you no longer need.".to_string(),
        risk_level: "low".to_string(),
        auto_runnable: false,
        guidance: vec![
          "Open System Settings → General → Storage.".to_string(),
          "Use the Recommendations panel to offload large files and empty trash.".to_string(),
          "Re-check storage usage after each cleanup pass.".to_string(),
        ],
      });

      steps.push(RemediationStep {
        id: "clear_user_caches".to_string(),
        title: "Clear user caches".to_string(),
        description: "~/Library/Caches holds app caches that macOS will rebuild on demand. Safe to remove the ones you no longer need open.".to_string(),
        risk_level: "low".to_string(),
        auto_runnable: false,
        guidance: vec![
          "Quit the apps whose caches you intend to clear.".to_string(),
          "In Finder, choose Go → Go to Folder… and enter ~/Library/Caches.".to_string(),
          "Move the relevant cache folders to Trash, then empty it.".to_string(),
        ],
      });
    }

    if has_process_pressure {
      steps.push(RemediationStep {
        id: "review_process_candidates".to_string(),
        title: "Review resource-heavy processes".to_string(),
        description: "Use Activity Monitor to identify processes with sustained high CPU or memory use. Quitting one user-facing app often resolves the warning.".to_string(),
        risk_level: "medium".to_string(),
        auto_runnable: false,
        guidance: vec![
          "Open Activity Monitor → CPU tab and sort by % CPU.".to_string(),
          "Identify non-critical apps with sustained high usage.".to_string(),
          "Quit them normally (Cmd+Q) — avoid force-killing system processes.".to_string(),
        ],
      });
    }

    if steps.is_empty() {
      steps.push(RemediationStep {
        id: "no_action_required".to_string(),
        title: "No action required".to_string(),
        description: "No risky or harmful issues were detected. Your system is running within healthy limits.".to_string(),
        risk_level: "low".to_string(),
        auto_runnable: false,
        guidance: vec![
          "Xclense will keep monitoring and notify you when something changes.".to_string(),
        ],
      });
    }

    let auto_safe_steps: Vec<String> = steps
      .iter()
      .filter(|step| step.auto_runnable)
      .map(|step| step.id.clone())
      .collect();

    RemediationPlan {
      generated_at_epoch_ms: now_epoch_ms(),
      issue_count: report.total_issues,
      steps,
      auto_safe_steps,
    }
  }

  fn run_one_remediation_step(step_id: &str) -> RemediationStepResult {
    let performed_at = now_epoch_ms();

    match step_id {
      "free_inactive_memory" => {
        // macOS `purge` is the supported way to request that the kernel
        // discard clean, inactive file-backed pages. It is documented as
        // safe for end users and does not affect running apps.
        let result = Command::new("/usr/bin/purge").output();

        match result {
          Ok(output) if output.status.success() => RemediationStepResult {
            step_id: step_id.to_string(),
            status: "succeeded".to_string(),
            message: "Inactive memory pages were released to the free pool.".to_string(),
            performed_at_epoch_ms: performed_at,
          },
          Ok(output) => RemediationStepResult {
            step_id: step_id.to_string(),
            status: "failed".to_string(),
            message: format!(
              "purge exited with status {:?}: {}",
              output.status.code(),
              String::from_utf8_lossy(&output.stderr).trim()
            ),
            performed_at_epoch_ms: performed_at,
          },
          Err(error) => RemediationStepResult {
            step_id: step_id.to_string(),
            status: "failed".to_string(),
            message: format!("unable to launch purge: {}", error),
            performed_at_epoch_ms: performed_at,
          },
        }
      }
      _ => RemediationStepResult {
        step_id: step_id.to_string(),
        status: "skipped".to_string(),
        message: "This step requires manual action — see the guidance in the overlay.".to_string(),
        performed_at_epoch_ms: performed_at,
      },
    }
  }

  #[tauri::command]
  pub fn get_remediation_plan(
    report: Option<AnalysisReport>,
  ) -> Result<RemediationPlan, String> {
    let resolved = match report {
      Some(value) => value,
      None => analyze_issues()?,
    };

    Ok(build_safe_remediation_plan(&resolved))
  }

  #[tauri::command]
  pub fn run_safe_remediation(
    app: tauri::AppHandle,
    step_ids: Vec<String>,
  ) -> Result<RemediationExecution, String> {
    if step_ids.is_empty() {
      return Err("no remediation steps were requested".to_string());
    }

    let plan = build_safe_remediation_plan(&analyze_issues()?);

    let mut results: Vec<RemediationStepResult> = Vec::new();
    let mut all_succeeded = true;

    for requested_id in &step_ids {
      let step = plan
        .steps
        .iter()
        .find(|candidate| candidate.id == *requested_id);

      let result = match step {
        Some(candidate) if candidate.auto_runnable => {
          let outcome = run_one_remediation_step(&candidate.id);
          if outcome.status != "succeeded" {
            all_succeeded = false;
          }
          outcome
        }
        Some(_) => RemediationStepResult {
          step_id: requested_id.clone(),
          status: "skipped".to_string(),
          message: "This step requires manual action — see the guidance in the overlay.".to_string(),
          performed_at_epoch_ms: now_epoch_ms(),
        },
        None => {
          all_succeeded = false;
          RemediationStepResult {
            step_id: requested_id.clone(),
            status: "unknown".to_string(),
            message: "step id did not match any known remediation step".to_string(),
            performed_at_epoch_ms: now_epoch_ms(),
          }
        }
      };

      // Mirror the audit pattern used by manage_process_action.
      let record = ActionAuditRecord {
        audit_id: format!("remediation-{}-{}", result.performed_at_epoch_ms, result.step_id),
        action: format!("remediate:{}", result.step_id),
        pid: 0,
        process_name: "system".to_string(),
        decision: result.status.clone(),
        decision_code: if result.status == "succeeded" {
          "REMEDIATION_EXECUTED".to_string()
        } else if result.status == "skipped" {
          "REMEDIATION_MANUAL".to_string()
        } else {
          "REMEDIATION_FAILED".to_string()
        },
        reason: result.message.clone(),
        risk_level: "low".to_string(),
        requested_at_epoch_ms: result.performed_at_epoch_ms,
        completed_at_epoch_ms: Some(now_epoch_ms()),
        source_version: env!("CARGO_PKG_VERSION").to_string(),
        source_context: Some("dashboard_fix_overlay".to_string()),
      };
      let _ = append_action_audit(&app, &record);

      results.push(result);
    }

    Ok(RemediationExecution {
      requested_step_ids: step_ids,
      results,
      all_succeeded,
    })
  }

  fn detect_disk_kind(disk: &str) -> (String, bool, bool) {
    let output = Command::new("diskutil")
      .args(["info", &format!("/dev/{}", disk)])
      .output();

    let mut kind = "Unknown".to_string();
    let mut removable = false;
    let mut internal = false;

    if let Ok(result) = output {
      if result.status.success() {
        let text = String::from_utf8_lossy(&result.stdout);
        let lower = text.to_lowercase();

        if lower.contains("nvme") {
          kind = "NVMe SSD".to_string();
        } else if lower.contains("apple ssd") || lower.contains("apple internal") {
          kind = "Apple Internal SSD".to_string();
        } else if lower.contains("fusion") {
          kind = "Fusion Drive".to_string();
        } else if lower.contains("solid state") || lower.contains(" ssd: yes") {
          kind = "SSD".to_string();
        } else if lower.contains("removable media: yes") || lower.contains("removable: yes") {
          removable = true;
          kind = "Removable".to_string();
        } else if lower.contains("rotational") {
          kind = "HDD (Rotational)".to_string();
        }

        removable = removable
          || lower.contains("removable media: yes")
          || lower.contains("ejectable: yes")
          || lower.contains("removable: yes");
        internal = lower.contains("internal: yes") || lower.contains("internal media");
      }
    }

    (kind, removable, internal)
  }

  fn physical_disk_size_bytes(disk: &str) -> u64 {
    let output = Command::new("diskutil")
      .args(["info", &format!("/dev/{}", disk)])
      .output();

    if let Ok(result) = output {
      if result.status.success() {
        let text = String::from_utf8_lossy(&result.stdout);
        if let Some(line) = text
          .lines()
          .find(|line| line.to_lowercase().contains("disk size"))
        {
          // Example: "Disk Size: 500.1 GB (500107862016 Bytes)"
          if let Some(start) = line.find('(') {
            if let Some(end) = line[start..].find(' ') {
              let candidate = &line[start + 1..start + end];
              if let Ok(value) = candidate.replace(',', "").parse::<u64>() {
                return value;
              }
            }
          }
        }
      }
    }

    0
  }

  fn list_physical_disks() -> Vec<PhysicalDisk> {
    let output = Command::new("diskutil").args(["list"]).output();

    let stdout = match output {
      Ok(value) if value.status.success() => String::from_utf8_lossy(&value.stdout).to_string(),
      _ => return Vec::new(),
    };

    let mut disks: Vec<PhysicalDisk> = Vec::new();

    for line in stdout.lines() {
      let trimmed = line.trim_start();
      if !trimmed.starts_with("/dev/disk") {
        continue;
      }

      let device = trimmed
        .split_whitespace()
        .next()
        .unwrap_or("")
        .trim_start_matches("/dev/")
        .to_string();

      if device.is_empty() {
        continue;
      }

      let (kind, removable, internal) = detect_disk_kind(&device);
      let size_bytes = physical_disk_size_bytes(&device);

      disks.push(PhysicalDisk {
        device: format!("/dev/{}", device),
        kind,
        size_bytes,
        removable,
        internal,
      });
    }

    disks.sort_by(|a, b| a.device.cmp(&b.device));
    disks
  }

  fn parse_df_volumes() -> Vec<VolumeInfo> {
    let output = match Command::new("df").args(["-k"]).output() {
      Ok(value) if value.status.success() => String::from_utf8_lossy(&value.stdout).to_string(),
      _ => return Vec::new(),
    };

    let mut volumes: Vec<VolumeInfo> = Vec::new();

    for (index, line) in output.lines().enumerate() {
      if index == 0 {
        continue;
      }

      let mut fields = line.split_whitespace();
      let filesystem = fields.next().unwrap_or("").to_string();
      let total_kb = match fields.next().and_then(|value| value.parse::<u64>().ok()) {
        Some(value) => value,
        None => continue,
      };
      let used_kb = match fields.next().and_then(|value| value.parse::<u64>().ok()) {
        Some(value) => value,
        None => continue,
      };
      let free_kb = match fields.next().and_then(|value| value.parse::<u64>().ok()) {
        Some(value) => value,
        None => continue,
      };
      let _percent_token = fields.next().unwrap_or("0%").trim_end_matches('%');
      let used_percent = if total_kb == 0 {
        0.0
      } else {
        (used_kb as f64 / total_kb as f64) * 100.0
      };
      let mount_point = fields.collect::<Vec<&str>>().join(" ");

      if filesystem.is_empty() || mount_point.is_empty() {
        continue;
      }

      let lower_fs = filesystem.to_lowercase();
      if lower_fs.starts_with("devfs")
        || lower_fs.starts_with("map")
        || lower_fs.starts_with("tmpfs")
        || lower_fs.starts_with("nullfs")
        || lower_fs.starts_with("union")
        || lower_fs.starts_with("autofs")
        || lower_fs.starts_with("securityfs")
      {
        continue;
      }

      if total_kb == 0 {
        continue;
      }

      volumes.push(VolumeInfo {
        mount_point,
        filesystem,
        total_bytes: total_kb.saturating_mul(1024),
        used_bytes: used_kb.saturating_mul(1024),
        free_bytes: free_kb.saturating_mul(1024),
        used_percent,
      });
    }

    volumes
  }

  fn shell_output_trimmed(command: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(command).args(args).output().ok()?;
    if !output.status.success() {
      return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
  }

  #[tauri::command]
  pub fn get_storage_detail() -> Result<StorageDetail, String> {
    let physical_disks = list_physical_disks();
    let volumes = parse_df_volumes();

    let summary = scan_storage().unwrap_or_else(|_| {
      let total_bytes: u64 = volumes.iter().map(|volume| volume.total_bytes).sum();
      let used_bytes: u64 = volumes.iter().map(|volume| volume.used_bytes).sum();
      let free_bytes: u64 = volumes.iter().map(|volume| volume.free_bytes).sum();
      let used_percent = if total_bytes == 0 {
        0.0
      } else {
        (used_bytes as f64 / total_bytes as f64) * 100.0
      };

      StorageSummary {
        total_bytes,
        used_bytes,
        free_bytes,
        used_percent,
        scanned_at_epoch_ms: now_epoch_ms(),
      }
    });

    let mac_model = shell_output_trimmed("sysctl", &["-n", "hw.model"])
      .or_else(|| {
        shell_output_trimmed("system_profiler", &["SPHardwareDataType"])
          .and_then(|value| {
            value
              .lines()
              .find(|line| line.contains("Model"))
              .map(|line| line.split(':').nth(1).unwrap_or("").trim().to_string())
          })
      })
      .unwrap_or_else(|| "Unknown Mac".to_string());

    let arch = match shell_output_trimmed("uname", &["-m"]) {
      Some(value) if value == "arm64" => "Apple Silicon (arm64)".to_string(),
      Some(value) if value == "x86_64" => "Intel 64-bit (x86_64)".to_string(),
      Some(value) => value,
      None => "Unknown".to_string(),
    };

    let macos_version =
      shell_output_trimmed("sw_vers", &["-productVersion"]).unwrap_or_else(|| "Unknown".to_string());

    Ok(StorageDetail {
      scanned_at_epoch_ms: now_epoch_ms(),
      mac_model,
      architecture: arch,
      macos_version,
      physical_disks,
      volumes,
      summary,
    })
  }

  fn default_storage_categories() -> Vec<StorageCategory> {
    vec![
      StorageCategory {
        id: "user_caches".to_string(),
        label: "User app caches".to_string(),
        description: "App cache folders under ~/Library/Caches. These are usually safe to clear when apps are closed.".to_string(),
        color: "sky".to_string(),
        path_prefixes: vec!["Library/Caches".to_string()],
        risk_level: "low".to_string(),
      },
      StorageCategory {
        id: "user_logs".to_string(),
        label: "User app logs".to_string(),
        description: "Diagnostic logs under ~/Library/Logs. Safe to remove for closed apps.".to_string(),
        color: "violet".to_string(),
        path_prefixes: vec!["Library/Logs".to_string()],
        risk_level: "low".to_string(),
      },
      StorageCategory {
        id: "downloads".to_string(),
        label: "Old downloads".to_string(),
        description: "Files in ~/Downloads. Review before cleanup because these may be user-created files or installers.".to_string(),
        color: "amber".to_string(),
        path_prefixes: vec!["Downloads".to_string()],
        risk_level: "medium".to_string(),
      },
      StorageCategory {
        id: "trash".to_string(),
        label: "Trash".to_string(),
        description: "Items currently in the user's Trash bin.".to_string(),
        color: "emerald".to_string(),
        path_prefixes: vec![".Trash".to_string()],
        risk_level: "medium".to_string(),
      },
      StorageCategory {
        id: "browser_cache".to_string(),
        label: "Browser caches".to_string(),
        description: "Safari/Chrome/Firefox cache data stored in containers and Library folders.".to_string(),
        color: "rose".to_string(),
        path_prefixes: vec![
          "Library/Containers/com.apple.Safari/Data/Library/Caches".to_string(),
          "Library/Caches/com.apple.Safari".to_string(),
          "Library/Caches/Google".to_string(),
          "Library/Caches/Firefox".to_string(),
          "Library/Application Support/Google/Chrome/Default/Cache".to_string(),
          "Library/Application Support/Google/Chrome/Default/Code Cache".to_string(),
          "Library/Application Support/Firefox/Profiles".to_string(),
        ],
        risk_level: "low".to_string(),
      },
      StorageCategory {
        id: "developer_artifacts".to_string(),
        label: "Developer artifacts".to_string(),
        description: "Xcode, simulator, build, and language package caches that can often be regenerated.".to_string(),
        color: "fuchsia".to_string(),
        path_prefixes: vec![
          "Library/Developer/Xcode/DerivedData".to_string(),
          "Library/Developer/Xcode/Archives".to_string(),
          "Library/Developer/Xcode/iOS DeviceSupport".to_string(),
          "Library/Developer/CoreSimulator".to_string(),
          "Library/Caches/CocoaPods".to_string(),
        ],
        risk_level: "low".to_string(),
      },
      StorageCategory {
        id: "package_manager_caches".to_string(),
        label: "Package manager caches".to_string(),
        description: "npm, Yarn, pnpm, pip, Gradle, Maven, Cargo, and Homebrew caches. These can usually be re-downloaded.".to_string(),
        color: "sky".to_string(),
        path_prefixes: vec![
          ".npm".to_string(),
          ".yarn/cache".to_string(),
          ".pnpm-store".to_string(),
          ".cache".to_string(),
          ".gradle/caches".to_string(),
          ".m2/repository".to_string(),
          ".cargo/registry".to_string(),
          "Library/Caches/pip".to_string(),
          "Library/Caches/Homebrew".to_string(),
        ],
        risk_level: "low".to_string(),
      },
      StorageCategory {
        id: "app_container_caches".to_string(),
        label: "App container caches".to_string(),
        description: "Cache folders discovered inside app Containers and Group Containers.".to_string(),
        color: "violet".to_string(),
        path_prefixes: vec![],
        risk_level: "low".to_string(),
      },
      StorageCategory {
        id: "node_modules".to_string(),
        label: "node_modules folders".to_string(),
        description: "Project dependency folders. Review before cleanup because projects need reinstalling dependencies after removal.".to_string(),
        color: "amber".to_string(),
        path_prefixes: vec![],
        risk_level: "medium".to_string(),
      },
      StorageCategory {
        id: "large_files".to_string(),
        label: "Large files".to_string(),
        description: "Large archives, disk images, videos, and installers found in common user folders. Review before cleanup.".to_string(),
        color: "rose".to_string(),
        path_prefixes: vec![],
        risk_level: "medium".to_string(),
      },
      StorageCategory {
        id: "system_temp".to_string(),
        label: "System temporary folders".to_string(),
        description: "Readable temporary folders under /tmp and /private. Some items may be active, so review carefully.".to_string(),
        color: "fuchsia".to_string(),
        path_prefixes: vec![
          "/tmp".to_string(),
          "/private/tmp".to_string(),
          "/private/var/tmp".to_string(),
        ],
        risk_level: "medium".to_string(),
      },
    ]
  }

  fn directory_size_bytes(path: &Path) -> u64 {
    let output = Command::new("du").args(["-sk", "-s"]).arg(path).output();
    match output {
      Ok(value) if value.status.success() => {
        let text = String::from_utf8_lossy(&value.stdout);
        let first = text.split_whitespace().next().unwrap_or("0");
        first.parse::<u64>().unwrap_or(0).saturating_mul(1024)
      }
      _ => 0,
    }
  }

  fn dir_modified_epoch_ms(path: &Path) -> u128 {
    fs::metadata(path)
      .and_then(|meta| meta.modified())
      .ok()
      .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
      .map(|duration| duration.as_millis())
      .unwrap_or(0)
  }

  fn dir_accessed_epoch_ms(path: &Path) -> u128 {
    fs::metadata(path)
      .and_then(|meta| meta.accessed())
      .ok()
      .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
      .map(|duration| duration.as_millis())
      .unwrap_or(0)
  }

  fn home_dir() -> Option<PathBuf> {
    if let Ok(value) = std::env::var("HOME") {
      if !value.is_empty() {
        return Some(PathBuf::from(value));
      }
    }

    let output = Command::new("sh").args(["-c", "echo $HOME"]).output().ok()?;
    if !output.status.success() {
      return None;
    }
    let trimmed = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if trimmed.is_empty() {
      None
    } else {
      Some(PathBuf::from(trimmed))
    }
  }

  fn category_by_id(categories: &[StorageCategory], id: &str) -> Option<StorageCategory> {
    categories.iter().find(|category| category.id == id).cloned()
  }

  fn resolve_scan_prefix(home: &Path, prefix: &str) -> PathBuf {
    let candidate = PathBuf::from(prefix);
    if candidate.is_absolute() {
      candidate
    } else {
      home.join(candidate)
    }
  }

  fn is_cline_protected_path(path: &Path) -> bool {
    path.components().any(|component| {
      let text = component.as_os_str().to_string_lossy();
      text.eq_ignore_ascii_case("cline") || text.eq_ignore_ascii_case(".cline")
    })
  }

  fn push_storage_item(
    items: &mut Vec<StorageScanItem>,
    item_counter: &mut u32,
    category: &StorageCategory,
    path: &Path,
    size_bytes: u64,
    recommendation: String,
  ) {
    let is_cline_protected = is_cline_protected_path(path);
    let risk_level = if is_cline_protected {
      "high".to_string()
    } else {
      category.risk_level.clone()
    };
    let recommendation = if is_cline_protected {
      format!(
        "Protected Cline directory item. Xclense will not remove this automatically; review it manually. {}",
        recommendation
      )
    } else {
      recommendation
    };

    *item_counter = item_counter.saturating_add(1);
    items.push(StorageScanItem {
      id: format!("scan-{}-{}", category.id, item_counter),
      category_id: category.id.clone(),
      path: path.to_string_lossy().to_string(),
      size_bytes,
      modified_epoch_ms: dir_modified_epoch_ms(path),
      last_accessed_epoch_ms: dir_accessed_epoch_ms(path),
      risk_level,
      recommendation,
    });
  }

  fn scan_direct_children(
    category: &StorageCategory,
    root: &Path,
    min_size_bytes: u64,
    items: &mut Vec<StorageScanItem>,
    scanned: &mut u32,
    item_counter: &mut u32,
    recommendation: &str,
  ) {
    if let Ok(entries) = fs::read_dir(root) {
      for entry in entries.flatten() {
        let path = entry.path();
        if !path.exists() {
          continue;
        }
        if entry.file_type().map(|file_type| file_type.is_symlink()).unwrap_or(false) {
          continue;
        }
        *scanned = scanned.saturating_add(1);
        let size_bytes = if path.is_dir() {
          directory_size_bytes(&path)
        } else {
          fs::metadata(&path).map(|metadata| metadata.len()).unwrap_or(0)
        };
        if size_bytes < min_size_bytes {
          continue;
        }
        push_storage_item(
          items,
          item_counter,
          category,
          &path,
          size_bytes,
          recommendation.to_string(),
        );
      }
    }
  }

  fn scan_single_path(
    category: &StorageCategory,
    path: &Path,
    min_size_bytes: u64,
    items: &mut Vec<StorageScanItem>,
    scanned: &mut u32,
    item_counter: &mut u32,
    recommendation: &str,
  ) {
    if !path.exists() {
      return;
    }
    *scanned = scanned.saturating_add(1);
    let size_bytes = directory_size_bytes(path);
    if size_bytes < min_size_bytes {
      return;
    }
    push_storage_item(
      items,
      item_counter,
      category,
      path,
      size_bytes,
      recommendation.to_string(),
    );
  }

  fn should_skip_deep_scan_dir(name: &str) -> bool {
    matches!(
      name,
      ".git"
        | ".svn"
        | ".hg"
        | "Library"
        | "Applications"
        | "node_modules"
        | ".Trash"
        | "Pictures Library.photoslibrary"
        | "Photos Library.photoslibrary"
    )
  }

  fn walk_for_node_modules(
    root: &Path,
    category: &StorageCategory,
    items: &mut Vec<StorageScanItem>,
    scanned: &mut u32,
    item_counter: &mut u32,
    seen: &mut std::collections::HashSet<String>,
    max_depth: usize,
    max_scanned: u32,
  ) {
    if max_depth == 0 || *scanned >= max_scanned || !root.is_dir() {
      return;
    }

    let entries = match fs::read_dir(root) {
      Ok(value) => value,
      Err(_) => return,
    };

    for entry in entries.flatten() {
      if *scanned >= max_scanned {
        break;
      }
      let path = entry.path();
      let file_type = match entry.file_type() {
        Ok(value) => value,
        Err(_) => continue,
      };
      if file_type.is_symlink() || !file_type.is_dir() {
        continue;
      }
      *scanned = scanned.saturating_add(1);
      let name = entry.file_name().to_string_lossy().to_string();
      if name == "node_modules" {
        let key = path.to_string_lossy().to_string();
        if seen.insert(key) {
          let size_bytes = directory_size_bytes(&path);
          if size_bytes >= 20 * 1024 * 1024 {
            push_storage_item(
              items,
              item_counter,
              category,
              &path,
              size_bytes,
              "Review this project dependency folder. Delete only if you can reinstall with npm, Yarn, or pnpm.".to_string(),
            );
          }
        }
        continue;
      }
      if should_skip_deep_scan_dir(&name) || name.ends_with(".app") || name.ends_with(".framework") {
        continue;
      }
      walk_for_node_modules(
        &path,
        category,
        items,
        scanned,
        item_counter,
        seen,
        max_depth - 1,
        max_scanned,
      );
    }
  }

  fn is_large_file_candidate(path: &Path) -> bool {
    let extension = path
      .extension()
      .and_then(|value| value.to_str())
      .unwrap_or("")
      .to_ascii_lowercase();
    matches!(
      extension.as_str(),
      "dmg" | "pkg" | "zip" | "tar" | "gz" | "xz" | "7z" | "rar" | "iso" | "mp4" | "mov" | "mkv" | "avi"
    )
  }

  fn walk_for_large_files(
    root: &Path,
    category: &StorageCategory,
    items: &mut Vec<StorageScanItem>,
    scanned: &mut u32,
    item_counter: &mut u32,
    seen: &mut std::collections::HashSet<String>,
    max_depth: usize,
    max_scanned: u32,
  ) {
    if max_depth == 0 || *scanned >= max_scanned || !root.is_dir() {
      return;
    }

    let entries = match fs::read_dir(root) {
      Ok(value) => value,
      Err(_) => return,
    };

    for entry in entries.flatten() {
      if *scanned >= max_scanned {
        break;
      }
      let path = entry.path();
      let file_type = match entry.file_type() {
        Ok(value) => value,
        Err(_) => continue,
      };
      if file_type.is_symlink() {
        continue;
      }
      *scanned = scanned.saturating_add(1);
      if file_type.is_file() {
        let size_bytes = fs::metadata(&path).map(|metadata| metadata.len()).unwrap_or(0);
        if size_bytes >= 500 * 1024 * 1024 && is_large_file_candidate(&path) {
          let key = path.to_string_lossy().to_string();
          if seen.insert(key) {
            push_storage_item(
              items,
              item_counter,
              category,
              &path,
              size_bytes,
              "Large archive, installer, or media file. Review before sending it to Trash.".to_string(),
            );
          }
        }
      } else if file_type.is_dir() {
        let name = entry.file_name().to_string_lossy().to_string();
        if should_skip_deep_scan_dir(&name) || name.ends_with(".app") || name.ends_with(".photoslibrary") {
          continue;
        }
        walk_for_large_files(
          &path,
          category,
          items,
          scanned,
          item_counter,
          seen,
          max_depth - 1,
          max_scanned,
        );
      }
    }
  }

  fn scan_app_container_caches(
    home: &Path,
    category: &StorageCategory,
    items: &mut Vec<StorageScanItem>,
    scanned: &mut u32,
    item_counter: &mut u32,
  ) {
    let roots = [home.join("Library/Containers"), home.join("Library/Group Containers")];
    for root in roots {
      if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
          let cache_path = entry.path().join("Data/Library/Caches");
          if !cache_path.exists() {
            continue;
          }
          *scanned = scanned.saturating_add(1);
          let size_bytes = directory_size_bytes(&cache_path);
          if size_bytes < 10 * 1024 * 1024 {
            continue;
          }
          push_storage_item(
            items,
            item_counter,
            category,
            &cache_path,
            size_bytes,
            "App container cache. Safe to clean after quitting the related app; macOS/app can regenerate it.".to_string(),
          );
        }
      }
    }
  }

  fn scan_storage_categories() -> StorageScanResult {
    let started_at = now_epoch_ms();
    let categories = default_storage_categories();
    let home = match home_dir() {
      Some(value) => value,
      None => {
        return StorageScanResult {
          started_at_epoch_ms: started_at,
          completed_at_epoch_ms: now_epoch_ms(),
          scanned_paths: 0,
          items: Vec::new(),
          categories,
          total_recoverable_bytes: 0,
        };
      }
    };

    let mut items: Vec<StorageScanItem> = Vec::new();
    let mut scanned: u32 = 0;
    let mut item_counter: u32 = 0;

    for category in &categories {
      if category.id == "app_container_caches"
        || category.id == "node_modules"
        || category.id == "large_files"
      {
        continue;
      }

      for prefix in &category.path_prefixes {
        let candidate = resolve_scan_prefix(&home, prefix);
        if !candidate.exists() {
          continue;
        }

        match category.id.as_str() {
          "downloads" => scan_direct_children(
            category,
            &candidate,
            20 * 1024 * 1024,
            &mut items,
            &mut scanned,
            &mut item_counter,
            "Review this download before cleanup. It may be an installer, archive, or user-created file.",
          ),
          "trash" => scan_direct_children(
            category,
            &candidate,
            5 * 1024 * 1024,
            &mut items,
            &mut scanned,
            &mut item_counter,
            "Already in Trash. Send it through cleanup when you are sure it is no longer needed.",
          ),
          "user_caches" => scan_direct_children(
            category,
            &candidate,
            10 * 1024 * 1024,
            &mut items,
            &mut scanned,
            &mut item_counter,
            "App cache folder. Safe to clean after quitting the related app; it can be regenerated.",
          ),
          "user_logs" => scan_direct_children(
            category,
            &candidate,
            5 * 1024 * 1024,
            &mut items,
            &mut scanned,
            &mut item_counter,
            "Old diagnostic logs. Safe to clean unless you need them for troubleshooting.",
          ),
          "browser_cache" => scan_single_path(
            category,
            &candidate,
            10 * 1024 * 1024,
            &mut items,
            &mut scanned,
            &mut item_counter,
            "Browser cache. Quit the browser first; it can rebuild these files automatically.",
          ),
          "developer_artifacts" => scan_direct_children(
            category,
            &candidate,
            20 * 1024 * 1024,
            &mut items,
            &mut scanned,
            &mut item_counter,
            "Developer build/simulator artifact. Review active projects first; most of this data can be regenerated.",
          ),
          "package_manager_caches" => scan_single_path(
            category,
            &candidate,
            20 * 1024 * 1024,
            &mut items,
            &mut scanned,
            &mut item_counter,
            "Package manager cache. Usually safe to clean; packages can be downloaded again later.",
          ),
          "system_temp" => scan_direct_children(
            category,
            &candidate,
            20 * 1024 * 1024,
            &mut items,
            &mut scanned,
            &mut item_counter,
            "Temporary system item. Review carefully and avoid cleaning files from currently running apps.",
          ),
          _ => scan_single_path(
            category,
            &candidate,
            10 * 1024 * 1024,
            &mut items,
            &mut scanned,
            &mut item_counter,
            "Review this reclaimable storage item before cleanup.",
          ),
        }
      }
    }

    if let Some(category) = category_by_id(&categories, "app_container_caches") {
      scan_app_container_caches(&home, &category, &mut items, &mut scanned, &mut item_counter);
    }

    if let Some(category) = category_by_id(&categories, "node_modules") {
      let mut seen = std::collections::HashSet::new();
      let roots = [
        home.join("Developer"),
        home.join("Projects"),
        home.join("Sites"),
        home.join("Documents"),
        home.join("Desktop"),
        home.join("Downloads"),
      ];
      for root in roots {
        walk_for_node_modules(
          &root,
          &category,
          &mut items,
          &mut scanned,
          &mut item_counter,
          &mut seen,
          8,
          80_000,
        );
      }
    }

    if let Some(category) = category_by_id(&categories, "large_files") {
      let mut seen = std::collections::HashSet::new();
      let roots = [
        home.join("Downloads"),
        home.join("Desktop"),
        home.join("Documents"),
        home.join("Movies"),
        home.join("Pictures"),
        home.join("Developer"),
        home.join("Projects"),
        home.join("Sites"),
      ];
      for root in roots {
        walk_for_large_files(
          &root,
          &category,
          &mut items,
          &mut scanned,
          &mut item_counter,
          &mut seen,
          7,
          80_000,
        );
      }
    }

    let mut seen_paths = std::collections::HashSet::new();
    items.retain(|item| seen_paths.insert(item.path.clone()));

    // Sort largest first.
    items.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    let total_recoverable_bytes: u64 = items.iter().map(|item| item.size_bytes).sum();

    StorageScanResult {
      started_at_epoch_ms: started_at,
      completed_at_epoch_ms: now_epoch_ms(),
      scanned_paths: scanned,
      items,
      categories,
      total_recoverable_bytes,
    }
  }

  #[tauri::command]
  pub fn scan_storage_for_cleanup() -> Result<StorageScanResult, String> {
    Ok(scan_storage_categories())
  }

  fn emit_cleanup_progress(app: &tauri::AppHandle, event: CleanupProgressEvent) {
    let _ = app.emit("storage-cleanup-progress", event);
  }

  fn move_to_trash(path: &Path) -> Result<(), String> {
    if !path.exists() {
      return Err(format!("path does not exist: {}", path.display()));
    }

    let path_text = path.to_string_lossy().to_string();
    let output = Command::new("osascript")
      .args([
        "-e",
        "on run argv",
        "-e",
        "set targetPath to POSIX file (item 1 of argv)",
        "-e",
        "tell application \"Finder\" to delete targetPath",
        "-e",
        "end run",
        &path_text,
      ])
      .output()
      .map_err(|error| format!("failed to invoke Finder trash command: {}", error))?;

    if output.status.success() {
      Ok(())
    } else {
      let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
      Err(format!(
        "Finder could not move item to Trash. Nothing was permanently deleted. {}",
        stderr
      ))
    }
  }

  #[tauri::command]
  pub fn cleanup_storage_items(
    app: tauri::AppHandle,
    request: CleanupRequest,
  ) -> Result<CleanupResult, String> {
    if request.item_ids.is_empty() {
      return Err("no items were requested for cleanup".to_string());
    }

    let scan = scan_storage_categories();
    let by_id: std::collections::HashMap<String, &StorageScanItem> =
      scan.items.iter().map(|item| (item.id.clone(), item)).collect();

    if !request.acknowledged_risk {
      let audit_id = format!("cleanup-denied-{}", now_epoch_ms());
      let record = ActionAuditRecord {
        audit_id: audit_id.clone(),
        action: "storage_cleanup".to_string(),
        pid: 0,
        process_name: "system".to_string(),
        decision: "denied".to_string(),
        decision_code: "CONFIRMATION_REQUIRED_OR_INVALID".to_string(),
        reason: "cleanup acknowledgement is required".to_string(),
        risk_level: "medium".to_string(),
        requested_at_epoch_ms: now_epoch_ms(),
        completed_at_epoch_ms: Some(now_epoch_ms()),
        source_version: env!("CARGO_PKG_VERSION").to_string(),
        source_context: Some("storage_page".to_string()),
      };
      let _ = append_action_audit(&app, &record);

      return Err("cleanup acknowledgement is required".to_string());
    }

    let performed_at = now_epoch_ms();
    let audit_id = format!("cleanup-{}-{}", performed_at, request.item_ids.len());
    let total_items = request.item_ids.len() as u32;
    let mut results: Vec<CleanupItemResult> = Vec::new();
    let mut total_reclaimed: u64 = 0;
    let mut all_succeeded = true;

    emit_cleanup_progress(
      &app,
      CleanupProgressEvent {
        audit_id: audit_id.clone(),
        phase: "started".to_string(),
        current: 0,
        total: total_items,
        item_id: None,
        path: None,
        status: None,
        message: format!("Starting cleanup for {} selected item(s)", total_items),
        reclaimed_bytes: 0,
      },
    );

    for (index, item_id) in request.item_ids.iter().enumerate() {
      let current = index as u32 + 1;
      let item = match by_id.get(item_id) {
        Some(value) => value,
        None => {
          all_succeeded = false;
          let message = "item id did not match any item in the latest scan".to_string();
          results.push(CleanupItemResult {
            item_id: item_id.clone(),
            path: String::new(),
            status: "unknown".to_string(),
            message: message.clone(),
            reclaimed_bytes: 0,
            performed_at_epoch_ms: performed_at,
          });
          emit_cleanup_progress(
            &app,
            CleanupProgressEvent {
              audit_id: audit_id.clone(),
              phase: "item_completed".to_string(),
              current,
              total: total_items,
              item_id: Some(item_id.clone()),
              path: None,
              status: Some("unknown".to_string()),
              message,
              reclaimed_bytes: total_reclaimed,
            },
          );
          continue;
        }
      };

      emit_cleanup_progress(
        &app,
        CleanupProgressEvent {
          audit_id: audit_id.clone(),
          phase: "item_started".to_string(),
          current,
          total: total_items,
          item_id: Some(item.id.clone()),
          path: Some(item.path.clone()),
          status: Some("running".to_string()),
          message: "Moving item to Trash…".to_string(),
          reclaimed_bytes: total_reclaimed,
        },
      );

      let path = PathBuf::from(&item.path);
      if is_cline_protected_path(&path) {
        all_succeeded = false;
        let message = "Skipped protected Cline directory item. Nothing was moved or deleted.".to_string();
        results.push(CleanupItemResult {
          item_id: item.id.clone(),
          path: item.path.clone(),
          status: "skipped".to_string(),
          message: message.clone(),
          reclaimed_bytes: 0,
          performed_at_epoch_ms: performed_at,
        });
        emit_cleanup_progress(
          &app,
          CleanupProgressEvent {
            audit_id: audit_id.clone(),
            phase: "item_completed".to_string(),
            current,
            total: total_items,
            item_id: Some(item.id.clone()),
            path: Some(item.path.clone()),
            status: Some("unknown".to_string()),
            message,
            reclaimed_bytes: total_reclaimed,
          },
        );
        continue;
      }

      let outcome = move_to_trash(&path);

      match outcome {
        Ok(_) => {
          total_reclaimed = total_reclaimed.saturating_add(item.size_bytes);
          results.push(CleanupItemResult {
            item_id: item.id.clone(),
            path: item.path.clone(),
            status: "succeeded".to_string(),
            message: "Item moved to Trash.".to_string(),
            reclaimed_bytes: item.size_bytes,
            performed_at_epoch_ms: performed_at,
          });
          emit_cleanup_progress(
            &app,
            CleanupProgressEvent {
              audit_id: audit_id.clone(),
              phase: "item_completed".to_string(),
              current,
              total: total_items,
              item_id: Some(item.id.clone()),
              path: Some(item.path.clone()),
              status: Some("succeeded".to_string()),
              message: "Item moved to Trash.".to_string(),
              reclaimed_bytes: total_reclaimed,
            },
          );
        }
        Err(error) => {
          all_succeeded = false;
          let message = format!("Failed to remove item: {}", error);
          results.push(CleanupItemResult {
            item_id: item.id.clone(),
            path: item.path.clone(),
            status: "failed".to_string(),
            message: message.clone(),
            reclaimed_bytes: 0,
            performed_at_epoch_ms: performed_at,
          });
          emit_cleanup_progress(
            &app,
            CleanupProgressEvent {
              audit_id: audit_id.clone(),
              phase: "item_completed".to_string(),
              current,
              total: total_items,
              item_id: Some(item.id.clone()),
              path: Some(item.path.clone()),
              status: Some("failed".to_string()),
              message,
              reclaimed_bytes: total_reclaimed,
            },
          );
        }
      }
    }

    emit_cleanup_progress(
      &app,
      CleanupProgressEvent {
        audit_id: audit_id.clone(),
        phase: "completed".to_string(),
        current: total_items,
        total: total_items,
        item_id: None,
        path: None,
        status: Some(if all_succeeded { "succeeded".to_string() } else { "failed".to_string() }),
        message: format!("Cleanup finished. Reclaimed {} bytes.", total_reclaimed),
        reclaimed_bytes: total_reclaimed,
      },
    );

    let record = ActionAuditRecord {
      audit_id: audit_id.clone(),
      action: "storage_cleanup".to_string(),
      pid: 0,
      process_name: "system".to_string(),
      decision: if all_succeeded { "executed".to_string() } else { "failed".to_string() },
      decision_code: if all_succeeded { "CLEANUP_EXECUTED".to_string() } else { "CLEANUP_PARTIAL_OR_FAILED".to_string() },
      reason: format!("reclaimed {} bytes across {} items", total_reclaimed, results.len()),
      risk_level: "medium".to_string(),
      requested_at_epoch_ms: performed_at,
      completed_at_epoch_ms: Some(now_epoch_ms()),
      source_version: env!("CARGO_PKG_VERSION").to_string(),
      source_context: Some("storage_page".to_string()),
    };
    let _ = append_action_audit(&app, &record);

    Ok(CleanupResult {
      requested_item_ids: request.item_ids,
      results,
      total_reclaimed_bytes: total_reclaimed,
      all_succeeded,
      performed_at_epoch_ms: performed_at,
      audit_id,
    })
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
      commands::list_process_action_audits,
      commands::get_remediation_plan,
      commands::run_safe_remediation,
      commands::get_storage_detail,
      commands::scan_storage_for_cleanup,
      commands::cleanup_storage_items
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
