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
  /// Human-readable name for what the item actually is.
  pub label: String,
  /// Tool or app that owns the data.
  pub owner: String,
  /// "file" or "directory".
  pub entry_kind: String,
  pub hidden: bool,
  /// True when the entry matched Xclense's known-entry catalogue.
  pub identified: bool,
  /// True when Xclense refuses to clean the item automatically.
  pub protected: bool,
  /// True when the owning tool recreates the data by itself.
  pub regenerates: bool,
  pub size_bytes: u64,
  pub modified_epoch_ms: u128,
  pub last_accessed_epoch_ms: u128,
  pub risk_level: String,
  /// 0-99, higher means safer to remove.
  pub safety_score: u8,
  /// What stops working if the item is missing.
  pub impact_if_removed: String,
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
  pub hidden_item_count: u32,
  pub protected_item_count: u32,
  /// False when macOS TCC would prompt for every protected folder we touch.
  pub full_disk_access: bool,
  /// Category ids skipped because they need Full Disk Access.
  pub skipped_categories: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PermissionStatus {
  pub full_disk_access: bool,
  /// Number of protected locations that stay unscanned without access.
  pub protected_location_count: u32,
  pub message: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StorageScanProgressEvent {
  pub scan_id: String,
  /// "started" | "category_started" | "path" | "item_found" | "completed"
  pub phase: String,
  pub category_id: Option<String>,
  pub category_label: Option<String>,
  pub current_path: Option<String>,
  pub completed_stages: u32,
  pub total_stages: u32,
  pub scanned_paths: u32,
  pub items_found: u32,
  pub reclaimable_bytes: u64,
  pub message: String,
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
  /// Share of RAM that cannot be reclaimed on demand.
  ///
  /// **Not** `(total - free) / total`. That counts inactive and cached pages as
  /// used, so an idle Mac with a warm file cache reports ~95% and looks
  /// identical to one that is genuinely dying. macOS reclaims inactive pages
  /// freely, so only wired + active + compressed represents real demand.
  pub memory_pressure_percent: f64,
  /// Kernel and driver memory. Never pageable, never reclaimable — the only
  /// way to release it is a restart. Above roughly a third of RAM this is the
  /// dominant problem and no cache-clearing remedy will touch it.
  pub memory_wired_bytes: u64,
  pub memory_active_bytes: u64,
  /// Reclaimable on demand. This is the only pool `purge` can release, so its
  /// size decides whether recommending `purge` is honest or theatre.
  pub memory_inactive_bytes: u64,
  pub memory_compressed_bytes: u64,
  pub swap_total_bytes: u64,
  pub swap_used_bytes: u64,
  pub swap_free_bytes: u64,
  pub swap_used_percent: f64,
  /// Cumulative since boot. A large value with swap near capacity means the
  /// machine is thrashing rather than merely having paged out once.
  pub swapouts: u64,
  pub load_average_1m: f64,
  pub load_average_5m: f64,
  pub load_average_15m: f64,
  pub scanned_at_epoch_ms: u128,
}

/// What is actually wrong with memory, as opposed to how bad it looks.
///
/// These demand different answers and were previously indistinguishable: every
/// one of them produced "free inactive memory", which only helps in the
/// `CachePressure` case and silently accomplishes nothing in the others.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MemoryFailureMode {
  /// Swap near capacity with heavy paging. Restart or shed load.
  SwapThrashing,
  /// Kernel/driver memory has ballooned. Only a restart releases it.
  WiredBloat,
  /// Large reclaimable cache — the one case where `purge` genuinely helps.
  CachePressure,
  /// A single process dominates. Quitting it is the fix.
  ProcessHog,
  /// No single offender; many mid-sized processes add up.
  DeathByAThousandCuts,
  Healthy,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MemoryDiagnosis {
  pub mode: MemoryFailureMode,
  pub headline: String,
  pub explanation: String,
  /// Bytes a remedy could realistically release. Zero means every available
  /// action is a no-op and saying so is the useful answer.
  pub reclaimable_bytes: u64,
  pub restart_required: bool,
  pub evidence: Vec<String>,
  pub suggested_action: String,
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

    metas.sort_by_key(|meta| std::cmp::Reverse(meta.created_at_epoch_ms));

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

    records.sort_by_key(|record| std::cmp::Reverse(record.requested_at_epoch_ms));

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

    // Every field, not just "Pages free". The previous single-field read made
    // wired and compressed memory invisible, which is exactly where a starved
    // machine's memory actually goes.
    let vm_field = |label: &str| -> u64 {
      vm_stat_output
        .lines()
        .find(|line| line.starts_with(label))
        .and_then(|line| line.split(':').nth(1))
        .map(|value| value.replace(['.', ','], ""))
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(0)
    };

    let free_pages = vm_field("Pages free");
    let active_pages = vm_field("Pages active");
    let inactive_pages = vm_field("Pages inactive");
    let wired_pages = vm_field("Pages wired down");
    let compressed_pages = vm_field("Pages occupied by compressor");
    let swapouts = vm_field("Swapouts");

    let memory_free_bytes = free_pages.saturating_mul(page_size);
    let memory_active_bytes = active_pages.saturating_mul(page_size);
    let memory_inactive_bytes = inactive_pages.saturating_mul(page_size);
    let memory_wired_bytes = wired_pages.saturating_mul(page_size);
    let memory_compressed_bytes = compressed_pages.saturating_mul(page_size);
    let memory_used_bytes = memory_total_bytes.saturating_sub(memory_free_bytes);

    // Only genuinely unreclaimable memory counts. Inactive pages are handed
    // back on demand, so including them (as `total - free` does) reports a
    // healthy machine with a warm cache as critical and makes the number
    // useless for telling the two apart.
    let committed_bytes = memory_wired_bytes
      .saturating_add(memory_active_bytes)
      .saturating_add(memory_compressed_bytes);
    let memory_pressure_percent = if memory_total_bytes == 0 {
      0.0
    } else {
      ((committed_bytes as f64 / memory_total_bytes as f64) * 100.0).min(100.0)
    };

    // `sysctl vm.swapusage` prints e.g.
    //   vm.swapusage: total = 14336.00M  used = 13252.44M  free = 1083.56M
    // Swap was previously never read at all, so a machine paging itself to
    // death looked the same as one that had never swapped.
    let parse_swap_field = |text: &str, label: &str| -> u64 {
      text
        .split(label)
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|token| {
          let (value, scale) = match token.chars().last() {
            Some('G') => (&token[..token.len() - 1], 1024.0 * 1024.0 * 1024.0),
            Some('M') => (&token[..token.len() - 1], 1024.0 * 1024.0),
            Some('K') => (&token[..token.len() - 1], 1024.0),
            _ => (token, 1.0),
          };
          value.parse::<f64>().ok().map(|number| (number * scale) as u64)
        })
        .unwrap_or(0)
    };

    let swap_output = run("sysctl", &["vm.swapusage"]).unwrap_or_default();
    let swap_total_bytes = parse_swap_field(&swap_output, "total =");
    let swap_used_bytes = parse_swap_field(&swap_output, "used =");
    let swap_free_bytes = parse_swap_field(&swap_output, "free =");
    let swap_used_percent = if swap_total_bytes == 0 {
      0.0
    } else {
      (swap_used_bytes as f64 / swap_total_bytes as f64) * 100.0
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
      memory_wired_bytes,
      memory_active_bytes,
      memory_inactive_bytes,
      memory_compressed_bytes,
      swap_total_bytes,
      swap_used_bytes,
      swap_free_bytes,
      swap_used_percent,
      swapouts,
      load_average_1m,
      load_average_5m,
      load_average_15m,
      scanned_at_epoch_ms: now_epoch_ms(),
    })
  }

  /// Wired memory above this share of RAM is the dominant problem, and nothing
  /// short of a restart releases it. Normal is roughly 15-25%.
  const WIRED_BLOAT_THRESHOLD_PERCENT: f64 = 35.0;
  /// Below this, releasing inactive pages is not worth recommending — it
  /// completes successfully and changes nothing the user can perceive.
  const PURGE_WORTH_IT_BYTES: u64 = 1024 * 1024 * 1024;

  /// Works out which memory problem the machine actually has.
  ///
  /// Ordered by which cause dominates, not by which is easiest to fix: swap
  /// exhaustion and wired bloat both make cache-clearing pointless, so they
  /// must be ruled out before `purge` is ever suggested.
  pub(crate) fn diagnose_memory(health: &SystemHealth, processes: &[ProcessInfo]) -> MemoryDiagnosis {
    let gb = |bytes: u64| bytes as f64 / 1_073_741_824.0;
    let wired_percent = if health.memory_total_bytes == 0 {
      0.0
    } else {
      (health.memory_wired_bytes as f64 / health.memory_total_bytes as f64) * 100.0
    };

    let mut evidence = vec![
      format!(
        "{:.2} GB RAM · {:.0}% committed (wired + active + compressed)",
        gb(health.memory_total_bytes),
        health.memory_pressure_percent
      ),
      format!(
        "wired {:.2} GB ({:.0}%) · active {:.2} GB · inactive {:.2} GB · compressed {:.2} GB",
        gb(health.memory_wired_bytes),
        wired_percent,
        gb(health.memory_active_bytes),
        gb(health.memory_inactive_bytes),
        gb(health.memory_compressed_bytes)
      ),
    ];
    if health.swap_total_bytes > 0 {
      evidence.push(format!(
        "swap {:.2} GB of {:.2} GB used ({:.0}%)",
        gb(health.swap_used_bytes),
        gb(health.swap_total_bytes),
        health.swap_used_percent
      ));
    }

    let biggest = processes
      .iter()
      .max_by(|a, b| a.memory_percent.total_cmp(&b.memory_percent));

    // Swap first: once the machine is paging heavily, every in-memory remedy
    // is beside the point.
    if health.swap_used_percent >= 80.0 {
      return MemoryDiagnosis {
        mode: MemoryFailureMode::SwapThrashing,
        headline: format!("Swap is {:.0}% full — the machine is paging to disk", health.swap_used_percent),
        explanation: format!(
          "{:.2} GB of {:.2} GB swap is in use. Everything is competing for memory that no longer exists, \
           which is why the system feels slow regardless of what you close. Freeing cache will not help: \
           macOS drains swap only as demand falls, and there is no command that clears it. Restarting is \
           the reliable fix; closing the heaviest apps first will reduce how much comes straight back.",
          gb(health.swap_used_bytes),
          gb(health.swap_total_bytes)
        ),
        reclaimable_bytes: 0,
        restart_required: true,
        evidence,
        suggested_action: "restart_to_reclaim_memory".to_string(),
      };
    }

    if wired_percent >= WIRED_BLOAT_THRESHOLD_PERCENT {
      return MemoryDiagnosis {
        mode: MemoryFailureMode::WiredBloat,
        headline: format!("{:.0}% of RAM is wired and cannot be reclaimed", wired_percent),
        explanation: format!(
          "{:.2} GB is held by the kernel and drivers. Wired memory is never paged out and never released \
           to applications, so no app you quit and no cache you clear will recover it. Typical is 15-25%. \
           A restart is the only thing that resets it.",
          gb(health.memory_wired_bytes)
        ),
        reclaimable_bytes: 0,
        restart_required: true,
        evidence,
        suggested_action: "restart_to_reclaim_memory".to_string(),
      };
    }

    if let Some(process) = biggest {
      if process.memory_percent >= 25.0 {
        return MemoryDiagnosis {
          mode: MemoryFailureMode::ProcessHog,
          headline: format!("{} is using {:.0}% of memory", process.name, process.memory_percent),
          explanation: format!(
            "A single process accounts for most of the pressure. Quitting or restarting {} (pid {}) \
             recovers the most memory for the least disruption.",
            process.name, process.pid
          ),
          reclaimable_bytes: ((process.memory_percent as f64 / 100.0)
            * health.memory_total_bytes as f64) as u64,
          restart_required: false,
          evidence,
          suggested_action: "review_process_candidates".to_string(),
        };
      }
    }

    if health.memory_inactive_bytes >= PURGE_WORTH_IT_BYTES {
      return MemoryDiagnosis {
        mode: MemoryFailureMode::CachePressure,
        headline: format!("{:.2} GB is cached and can be released", gb(health.memory_inactive_bytes)),
        explanation:
          "Inactive memory is file cache the kernel hands back on demand. Releasing it is safe and \
           genuinely frees this much, though macOS would also reclaim it automatically under pressure."
            .to_string(),
        reclaimable_bytes: health.memory_inactive_bytes,
        restart_required: false,
        evidence,
        suggested_action: "free_inactive_memory".to_string(),
      };
    }

    if health.memory_pressure_percent >= 80.0 {
      let notable = processes.iter().filter(|p| p.memory_percent >= 2.0).count();
      return MemoryDiagnosis {
        mode: MemoryFailureMode::DeathByAThousandCuts,
        headline: "No single process is responsible".to_string(),
        explanation: format!(
          "Memory is committed but no process dominates — {notable} are each holding a noticeable share. \
           There is nothing to free: only {:.2} GB is cached, so releasing it changes little. Closing \
           several of the largest, or restarting, is what actually helps.",
          gb(health.memory_inactive_bytes)
        ),
        reclaimable_bytes: health.memory_inactive_bytes,
        restart_required: false,
        evidence,
        suggested_action: "review_process_candidates".to_string(),
      };
    }

    MemoryDiagnosis {
      mode: MemoryFailureMode::Healthy,
      headline: "Memory is healthy".to_string(),
      explanation: "Committed memory is within normal range and swap is not under pressure.".to_string(),
      reclaimable_bytes: health.memory_inactive_bytes,
      restart_required: false,
      evidence,
      suggested_action: "no_action_required".to_string(),
    }
  }

  #[tauri::command]
  pub fn diagnose_memory_condition() -> Result<MemoryDiagnosis, String> {
    let health = get_system_health()?;
    let processes = list_processes()?;
    Ok(diagnose_memory(&health, &processes))
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

    // Driven by the diagnosis rather than by the pressure number alone, so the
    // recommendation matches the actual cause. Reporting "free inactive
    // memory" against swap exhaustion produced a step that succeeded and
    // achieved nothing.
    let memory = diagnose_memory(&health, &processes);
    if memory.mode != MemoryFailureMode::Healthy {
      let severity = match memory.mode {
        MemoryFailureMode::SwapThrashing | MemoryFailureMode::WiredBloat => "critical",
        _ if health.memory_pressure_percent >= 90.0 => "critical",
        _ => "warning",
      };
      issues.push(IssueReport {
        id: "memory-pressure-001".to_string(),
        title: memory.headline.clone(),
        severity: severity.to_string(),
        confidence: 0.95,
        evidence: memory.evidence.clone(),
        recommendation: memory.explanation.clone(),
        suggested_action: memory.suggested_action.clone(),
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
      // Advisory only. macOS offers no way for an app to release wired memory
      // or drain swap, so the honest response is to say a restart is needed
      // rather than run something that reports success and changes nothing.
      "restart_to_reclaim_memory" => RemediationStepResult {
        step_id: step_id.to_string(),
        status: "manual".to_string(),
        message:
          "Restart required. Wired memory and swap cannot be released by any command — macOS frees \
           them only on reboot. Save your work and restart to reclaim this memory."
            .to_string(),
        performed_at_epoch_ms: performed_at,
      },
      "free_inactive_memory" => {
        // Only worth running when there is enough inactive memory for the
        // result to be perceptible. Below the threshold this succeeds and
        // frees a rounding error, which reads as a fix that did nothing.
        match get_system_health() {
          Ok(health) if health.memory_inactive_bytes < PURGE_WORTH_IT_BYTES => {
            return RemediationStepResult {
              step_id: step_id.to_string(),
              status: "skipped".to_string(),
              message: format!(
                "Skipped: only {:.2} GB is cached, so releasing it would not measurably help. \
                 Memory pressure here is coming from somewhere purge cannot reach.",
                health.memory_inactive_bytes as f64 / 1_073_741_824.0
              ),
              performed_at_epoch_ms: performed_at,
            };
          }
          _ => {}
        }

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
          ".yarn/berry/cache".to_string(),
          ".pnpm-store".to_string(),
          ".bun/install/cache".to_string(),
          ".deno".to_string(),
          ".cache".to_string(),
          ".gradle/caches".to_string(),
          ".m2/repository".to_string(),
          ".cargo/registry".to_string(),
          ".cocoapods/repos".to_string(),
          ".composer/cache".to_string(),
          ".gem".to_string(),
          "Library/Caches/pip".to_string(),
          "Library/Caches/Homebrew".to_string(),
          "Library/Caches/go-build".to_string(),
        ],
        risk_level: "low".to_string(),
      },
      StorageCategory {
        id: "hidden_home".to_string(),
        label: "Hidden home items".to_string(),
        description: "Every dot-file and dot-folder directly inside your home folder, with the owning tool and the impact of removing it.".to_string(),
        color: "emerald".to_string(),
        path_prefixes: vec![],
        risk_level: "medium".to_string(),
      },
      StorageCategory {
        id: "hidden_support".to_string(),
        label: "Hidden support data".to_string(),
        description: "Dot-folders hidden inside ~/Library and ~/Library/Application Support that normal Finder views never show.".to_string(),
        color: "violet".to_string(),
        path_prefixes: vec![],
        risk_level: "medium".to_string(),
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
        id: "app_support_data".to_string(),
        label: "App support data".to_string(),
        description: "Large per-app folders in ~/Library/Application Support. Removing one resets that app's local state and sign-in.".to_string(),
        color: "amber".to_string(),
        path_prefixes: vec!["Library/Application Support".to_string()],
        risk_level: "medium".to_string(),
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
        id: "project_build_caches".to_string(),
        label: "Project build caches".to_string(),
        description: "Hidden build and output folders inside projects (.next, .turbo, target, dist, __pycache__, …). Toolchains rebuild them on the next build.".to_string(),
        color: "fuchsia".to_string(),
        path_prefixes: vec![],
        risk_level: "low".to_string(),
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

  #[derive(Clone)]
  struct EntryProfile {
    label: &'static str,
    owner: &'static str,
    risk: &'static str,
    regenerates: bool,
    impact: &'static str,
    recommendation: &'static str,
  }

  fn profile(
    label: &'static str,
    owner: &'static str,
    risk: &'static str,
    regenerates: bool,
    impact: &'static str,
    recommendation: &'static str,
  ) -> EntryProfile {
    EntryProfile { label, owner, risk, regenerates, impact, recommendation }
  }

  /// Folders produced by a toolchain that are rebuilt on the next build.
  fn build_cache_dir_name(name: &str) -> bool {
    matches!(
      name,
      ".next"
        | ".turbo"
        | ".nuxt"
        | ".svelte-kit"
        | ".angular"
        | ".astro"
        | ".docusaurus"
        | ".parcel-cache"
        | ".vite"
        | ".webpack"
        | ".rollup.cache"
        | ".expo"
        | ".dart_tool"
        | ".gradle"
        | ".cxx"
        | ".terraform"
        | ".pytest_cache"
        | ".mypy_cache"
        | ".ruff_cache"
        | ".tox"
        | ".nyc_output"
        | ".ipynb_checkpoints"
        | "__pycache__"
        | ".venv"
        | "venv"
        | "DerivedData"
        | "Pods"
    )
  }

  /// Known dot-entries and cache folders, with the real-world impact of removing them.
  fn known_entry_profile(name: &str) -> Option<EntryProfile> {
    let entry = match name {
      // Package manager caches.
      ".npm" => profile("npm cache", "npm", "low", true, "npm re-downloads packages the next time you install; only that first install is slower.", "Package manager cache. Usually safe to clean; packages can be downloaded again later."),
      ".yarn" => profile("Yarn home", "Yarn", "medium", true, "Yarn's global cache and Berry releases are removed; Yarn re-downloads them, but pinned releases must be re-fetched.", "Contains Yarn's global cache plus release binaries. Safe if you have network access."),
      ".pnpm-store" => profile("pnpm content store", "pnpm", "low", true, "pnpm re-downloads packages; existing projects keep working because links are re-created on install.", "Package manager store. Run `pnpm install` afterwards in active projects."),
      ".cache" => profile("Shared tool cache", "CLI tools", "low", true, "Tools such as Yarn, Puppeteer, Playwright, and Hugging Face re-download their cached assets on demand.", "Shared cache folder used by many CLI tools. Safe to clean."),
      ".bun" => profile("Bun runtime + cache", "Bun", "medium", false, "Removes the bun binary itself along with its install cache; you must reinstall Bun.", "Contains the Bun binary. Clean only the install cache inside unless you plan to reinstall Bun."),
      ".deno" => profile("Deno cache", "Deno", "low", true, "Deno re-downloads remote modules on the next run.", "Remote module cache. Safe to clean."),
      ".nvm" => profile("nvm Node versions", "nvm", "medium", false, "Every Node.js version installed through nvm is deleted; `node` and `npm` stop working until you reinstall a version.", "Holds installed Node.js versions. Reinstall with `nvm install` after cleaning."),
      ".cargo" => profile("Cargo home", "Rust / Cargo", "medium", false, "Deletes installed cargo binaries plus the crates registry; rebuilds re-download crates and `cargo install` tools must be reinstalled.", "Prefer cleaning only ~/.cargo/registry instead of the whole folder."),
      ".rustup" => profile("Rust toolchains", "rustup", "medium", false, "All installed Rust toolchains are removed; `cargo` and `rustc` stop working until you run `rustup toolchain install`.", "Holds Rust toolchains. Clean only if you can reinstall them."),
      ".gradle" => profile("Gradle home", "Gradle", "low", true, "Gradle re-downloads dependencies and re-creates its daemon state on the next build.", "Gradle cache and daemon data. Safe to clean between builds."),
      ".m2" => profile("Maven repository", "Maven", "low", true, "Maven re-downloads every dependency on the next build, which makes that build much slower.", "Local Maven repository. Safe but the next build re-downloads everything."),
      ".gem" | ".rbenv" | ".rvm" => profile("Ruby toolchain data", "Ruby", "medium", false, "Installed Ruby versions and gems are removed; Ruby projects fail until you reinstall them.", "Ruby versions and gems. Reinstall before running Ruby projects again."),
      ".pyenv" | ".conda" | ".anaconda3" | ".miniconda3" | ".virtualenvs" => profile("Python toolchain data", "Python", "medium", false, "Installed Python versions and virtual environments are removed; scripts using them stop working until recreated.", "Python runtimes/environments. Recreate environments after cleaning."),
      ".composer" => profile("Composer cache", "Composer", "low", true, "Composer re-downloads PHP packages on the next install.", "PHP package cache. Safe to clean."),
      ".cocoapods" => profile("CocoaPods cache", "CocoaPods", "low", true, "CocoaPods re-clones the spec repos on the next `pod install`.", "Spec repos and cache. Safe to clean; the next `pod install` is slower."),
      ".swiftpm" => profile("Swift Package Manager cache", "Swift", "low", true, "Swift re-resolves and re-downloads packages on the next build.", "Swift package cache. Safe to clean."),
      ".android" => profile("Android SDK data", "Android Studio", "medium", false, "Emulator images (AVDs) and SDK settings are deleted; Android Studio must re-download the SDK and you lose emulator state.", "Android SDK/AVD data. Large but expensive to re-download."),
      ".gradle.properties" => profile("Gradle settings", "Gradle", "high", false, "Build credentials and JVM tuning for Gradle are lost.", "Gradle configuration file. Keep it."),

      // Editors, AI tools, apps.
      ".vscode" | ".vscode-insiders" => profile("VS Code user data", "VS Code", "medium", false, "Extensions installed for your user are removed and must be reinstalled; settings sync can restore them.", "VS Code extensions/state. Reinstall extensions after cleaning."),
      ".cursor" | ".windsurf" | ".continue" | ".codeium" | ".aider" => profile("AI editor data", "AI coding tool", "medium", false, "Extensions, local indexes, and chat history for the tool are removed.", "AI editor data and local indexes. Removing it resets the tool."),
      ".claude" => profile("Claude Code data", "Claude Code", "medium", false, "Claude Code settings, project memory, and session history are lost; the CLI keeps working with defaults.", "Claude Code configuration and history. Back it up before cleaning."),
      ".gemini" | ".codex" | ".copilot" => profile("AI CLI data", "AI CLI tool", "medium", false, "Sign-in state, settings, and cached session history for the CLI are lost; you must authenticate again.", "AI CLI configuration and history. You will need to sign in again."),
      ".zed" => profile("Zed editor data", "Zed", "medium", false, "Editor extensions, local state, and cached language servers are removed.", "Zed user data. Extensions are re-downloaded on next launch."),
      ".hermes" => profile("Hermes build cache", "React Native / Hermes", "low", true, "Hermes re-downloads or rebuilds its compiler artifacts on the next React Native build.", "Hermes engine artifacts. Safe to clean; next build is slower."),
      ".huggingface" => profile("Hugging Face cache", "Hugging Face", "medium", true, "Downloaded models and datasets are deleted and re-downloaded on demand, which can be many GB.", "Model/dataset cache. Safe but expensive to re-download."),
      ".jupyter" | ".ipython" => profile("Notebook runtime data", "Jupyter / IPython", "medium", true, "Kernel settings, saved sessions, and REPL history are lost; Jupyter recreates defaults.", "Notebook configuration and history. Mostly regenerated."),
      ".tldrc" | ".zcompdump" | ".zcompcache" | ".sass-cache" => profile("Shell helper cache", "Shell tooling", "low", true, "The tool rebuilds the cache the next time it runs.", "Regenerated helper cache. Safe to clean."),
      ".ollama" => profile("Ollama models", "Ollama", "medium", false, "Every downloaded model is deleted and must be pulled again (often many GB per model).", "Downloaded LLM models. Very large, but re-downloading costs bandwidth."),
      ".lmstudio" | ".cache/lm-studio" => profile("LM Studio models", "LM Studio", "medium", false, "Downloaded local models are deleted and must be re-downloaded.", "Local model store. Large; re-download costs bandwidth."),
      ".docker" => profile("Docker CLI config", "Docker", "high", false, "Registry credentials and CLI context configuration are lost; `docker push/pull` to private registries fails until you log in again.", "Holds registry credentials. Keep it; clean Docker disk images from Docker Desktop instead."),
      ".colima" | ".lima" | ".minikube" | ".vagrant.d" | ".podman" => profile("Local VM/cluster data", "Container tooling", "medium", false, "Local VMs or Kubernetes clusters and their disks are destroyed; you must recreate them.", "Local VM/cluster state. Recreate the environment after cleaning."),
      ".terraform.d" => profile("Terraform plugin cache", "Terraform", "medium", true, "Provider plugins re-download on the next `terraform init`; stored credentials for Terraform Cloud are lost.", "Plugin cache plus credentials. Check for a credentials file first."),
      ".oh-my-zsh" | ".antigen" | ".zprezto" | ".zinit" => profile("Zsh framework", "Zsh", "medium", false, "Your prompt, themes, and shell plugins stop working until the framework is reinstalled.", "Shell framework. Reinstall before opening a new terminal."),
      ".expo" => profile("Expo cache", "Expo", "low", true, "Expo rebuilds its cache on the next start.", "Build cache. Safe to clean."),
      ".electron" | ".electron-gyp" | ".node-gyp" => profile("Native build toolchain cache", "Node native builds", "low", true, "Headers and prebuilt binaries re-download the next time a native module is compiled.", "Native build cache. Safe to clean."),
      ".cups" | ".fontconfig" => profile("System helper cache", "macOS", "low", true, "The service rebuilds this automatically.", "Helper cache. Safe to clean."),
      ".Trash" => profile("Trash", "Finder", "medium", false, "Items already in the Trash are permanently removed.", "Empty only when you are sure nothing here is needed."),
      ".DS_Store" => profile("Finder view settings", "Finder", "low", true, "Finder recreates it; the folder loses its custom icon positions and view options.", "Finder metadata. Safe to remove."),
      ".CFUserTextEncoding" => profile("Text encoding preference", "macOS", "medium", true, "macOS recreates it at next login; a wrong locale can briefly affect terminal encoding.", "Tiny macOS preference file. Not worth cleaning."),

      // Histories — safe but you lose recall.
      ".zsh_history" | ".bash_history" | ".python_history" | ".node_repl_history" | ".psql_history"
      | ".sqlite_history" | ".lesshst" | ".viminfo" | ".wget-hsts" => profile("Shell/tool history", "Shell", "medium", true, "Only your command history is lost — nothing stops working, but past commands can no longer be recalled.", "Command history. Safe to remove if you do not need past commands."),

      // Credentials and configuration — never auto-clean.
      ".ssh" => profile("SSH keys", "OpenSSH", "high", false, "Private keys and known-hosts are destroyed; Git over SSH and every server login using these keys stops working and cannot be recovered.", "Never clean automatically. Back up before touching this folder."),
      ".gnupg" => profile("GPG keyring", "GnuPG", "high", false, "Private GPG keys are destroyed; signed commits and encrypted files become unrecoverable.", "Never clean automatically. Back up your keyring first."),
      ".aws" | ".azure" | ".gcloud" | ".oci" => profile("Cloud credentials", "Cloud CLI", "high", false, "Access keys and CLI profiles are removed; deployments and CLI commands fail until you re-authenticate.", "Contains cloud credentials. Keep it."),
      ".kube" => profile("Kubernetes config", "kubectl", "high", false, "Cluster contexts and certificates are removed; `kubectl` loses access to every cluster.", "Cluster credentials. Keep it."),
      ".config" => profile("XDG config root", "Many CLI tools", "high", false, "Shared configuration for dozens of CLI tools (gh, nvim, raycast, …) is lost, including some tokens.", "Configuration root. Clean individual subfolders instead."),
      ".local" => profile("User binaries and data", "User installs", "high", false, "User-installed executables in ~/.local/bin and app data in ~/.local/share are removed; pipx/uv tools stop working.", "Holds user-installed programs. Clean specific subfolders instead."),
      ".zshrc" | ".zprofile" | ".zshenv" | ".zlogin" | ".bashrc" | ".bash_profile" | ".profile"
      | ".inputrc" | ".vimrc" | ".tmux.conf" | ".curlrc" | ".wgetrc" | ".editorconfig" => profile("Shell/tool configuration", "Shell", "high", false, "Your shell or tool loses its configuration: PATH entries, aliases, and environment variables stop being applied in new sessions.", "Configuration file. Keep it; it reclaims almost no space anyway."),
      ".gitconfig" | ".gitignore_global" | ".git-credentials" | ".netrc" | ".npmrc" | ".yarnrc"
      | ".yarnrc.yml" | ".pypirc" | ".gemrc" => profile("Tool credentials/config", "Git & package tools", "high", false, "Identity, registry tokens, and auth settings are lost; pushes and private-registry installs start failing.", "Holds credentials or identity. Keep it."),

      _ => return None,
    };
    Some(entry)
  }

  fn category_default_profile(category_id: &str) -> EntryProfile {
    match category_id {
      "user_caches" => profile("App cache", "Installed app", "low", true, "The app rebuilds this cache on next launch; the first launch is slower and some previews reload.", "App cache folder. Safe to clean after quitting the related app."),
      "user_logs" => profile("App logs", "Installed app", "low", true, "Only diagnostic history is lost; apps recreate log files as they run.", "Old diagnostic logs. Safe unless you need them for troubleshooting."),
      "browser_cache" => profile("Browser cache", "Web browser", "low", true, "The browser re-downloads cached pages and images; sessions and bookmarks are untouched.", "Browser cache. Quit the browser first; it rebuilds automatically."),
      "developer_artifacts" => profile("Developer artifact", "Xcode / dev tools", "low", true, "Builds and simulator data are recreated on the next build; the next build is slower.", "Developer build artifact. Most of this data can be regenerated."),
      "package_manager_caches" => profile("Package manager cache", "Package manager", "low", true, "Packages are re-downloaded on the next install.", "Package manager cache. Usually safe to clean."),
      "app_container_caches" => profile("Sandboxed app cache", "Sandboxed app", "low", true, "The app regenerates its cache after you reopen it.", "App container cache. Safe to clean after quitting the app."),
      "app_support_data" => profile("App support data", "Installed app", "medium", false, "The app is reset to a fresh state: local databases, sign-in, and offline content are gone (cloud data can re-sync).", "App data folder — not a cache. Sign-in and local state are lost."),
      "node_modules" => profile("Project dependencies", "Node project", "medium", true, "The project cannot build or run until `npm install` (or yarn/pnpm) is run again.", "Project dependency folder. Delete only if you can reinstall."),
      "project_build_caches" => profile("Project build output", "Project toolchain", "low", true, "The toolchain regenerates it on the next build; only build time is lost.", "Build output folder. Safe to clean; rebuild afterwards."),
      "downloads" => profile("Downloaded file", "You", "medium", false, "The file is gone from Downloads; installers and archives must be downloaded again, and user-created files may be unrecoverable.", "Review this download before cleanup."),
      "trash" => profile("Trashed item", "Finder", "medium", false, "The item leaves the Trash permanently.", "Already in Trash. Remove when you are sure it is not needed."),
      "large_files" => profile("Large file", "You", "medium", false, "The file is removed from disk; media and installers must be re-obtained.", "Large archive, installer, or media file. Review carefully."),
      "system_temp" => profile("Temporary item", "macOS / apps", "medium", true, "Usually recreated automatically, but a running app can lose in-flight work.", "Temporary system item. Avoid cleaning files from running apps."),
      "hidden_home" => profile("Hidden home item", "Unidentified tool", "medium", false, "An unrecognised tool owns this folder; it may lose settings, cached data, or credentials.", "Unrecognised hidden item — open it before cleaning."),
      "hidden_support" => profile("Hidden support data", "Installed app", "medium", false, "Hidden per-app state is removed; the owning app may reset or re-sync.", "Hidden app data folder. Review before cleaning."),
      _ => profile("Storage item", "Unknown", "medium", false, "Impact is unknown; review the path before removing it.", "Review this reclaimable storage item before cleanup."),
    }
  }

  /// Paths Xclense refuses to clean automatically, with the reason shown in the UI.
  fn protected_path_reason(path: &Path) -> Option<String> {
    for component in path.components() {
      let text = component.as_os_str().to_string_lossy().to_string();
      if text.eq_ignore_ascii_case("cline") || text.eq_ignore_ascii_case(".cline") {
        return Some(
          "Protected Cline directory item. Xclense will not remove this automatically.".to_string(),
        );
      }
      let reason = match text.as_str() {
        ".ssh" => "Protected: SSH private keys and known-hosts.",
        ".gnupg" => "Protected: GPG keyring and private keys.",
        ".aws" | ".azure" | ".gcloud" | ".oci" => "Protected: cloud CLI credentials.",
        ".kube" => "Protected: Kubernetes cluster credentials.",
        ".docker" => "Protected: Docker registry credentials and contexts.",
        ".config" => "Protected: shared configuration root for many CLI tools.",
        ".local" => "Protected: user-installed binaries and application data.",
        "Keychains" => "Protected: macOS keychain data.",
        "MobileSync" => "Protected: iOS device backups.",
        _ => "",
      };
      if !reason.is_empty() {
        return Some(format!("{} Xclense will not remove it automatically.", reason));
      }
    }

    let name = path.file_name()?.to_string_lossy().to_string();
    let is_protected_file = matches!(
      name.as_str(),
      ".zshrc"
        | ".zprofile"
        | ".zshenv"
        | ".zlogin"
        | ".bashrc"
        | ".bash_profile"
        | ".profile"
        | ".inputrc"
        | ".vimrc"
        | ".tmux.conf"
        | ".curlrc"
        | ".wgetrc"
        | ".editorconfig"
        | ".gitconfig"
        | ".gitignore_global"
        | ".git-credentials"
        | ".netrc"
        | ".npmrc"
        | ".yarnrc"
        | ".yarnrc.yml"
        | ".pypirc"
        | ".gemrc"
        | ".gradle.properties"
    );
    if is_protected_file {
      return Some(
        "Protected: configuration or credential file. Xclense will not remove it automatically."
          .to_string(),
      );
    }
    None
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

  fn entry_size_bytes(path: &Path, is_dir: bool) -> u64 {
    if is_dir {
      directory_size_bytes(path)
    } else {
      fs::metadata(path).map(|metadata| metadata.len()).unwrap_or(0)
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

  /// Detects Full Disk Access by reading a path only FDA-granted apps can open.
  ///
  /// macOS 14+ guards `~/Library/Containers`, `~/Library/Group Containers`, and
  /// per-app folders in `~/Library/Application Support` behind the "access data
  /// from other apps" consent prompt, and it fires **once per app container**.
  /// With 742 containers on a typical machine that is 742 dialogs, so anything
  /// touching those paths must be skipped until this returns true.
  fn has_full_disk_access() -> bool {
    let home = match home_dir() {
      Some(value) => value,
      None => return false,
    };
    // Reading TCC.db itself requires Full Disk Access and never prompts —
    // it fails silently when access is not granted.
    fs::File::open(home.join("Library/Application Support/com.apple.TCC/TCC.db")).is_ok()
  }

  /// Every location macOS guards behind a consent dialog.
  ///
  /// Two different TCC services, same consequence:
  ///
  /// * **App data** — `Library/Containers`, `Library/Group Containers`,
  ///   `Library/Application Support`. One dialog **per app**, so roughly a
  ///   thousand on a normal machine.
  /// * **Personal folders** — Desktop, Documents, Downloads, Movies, Pictures.
  ///   One dialog each.
  ///
  /// Unlike the Full Disk Access probe there is no silent way to test these:
  /// attempting the read *is* what raises the dialog.
  pub(crate) const CONSENT_GUARDED_ROOTS: [&str; 8] = [
    "Library/Containers",
    "Library/Group Containers",
    "Library/Application Support",
    "Desktop",
    "Documents",
    "Downloads",
    "Movies",
    "Pictures",
  ];

  /// Whether reading `path` would raise a consent dialog.
  ///
  /// This is deliberately a check on the **path**, not on the category asking
  /// for it. Gating by category has now failed twice, because guarded paths
  /// hide inside category definitions — `browser_cache` reads
  /// `Library/Containers/com.apple.Safari/…` and
  /// `Library/Application Support/Google/Chrome/…`, so a category-level
  /// allowlist waved it straight through. Anything that reads the filesystem
  /// must ask here, and a new scan location cannot bypass it by construction.
  fn is_consent_guarded(path: &Path, home: &Path) -> bool {
    CONSENT_GUARDED_ROOTS
      .iter()
      .any(|relative| path.starts_with(home.join(relative)))
  }

  /// Drops the consent-guarded roots from a walk list unless access is held.
  fn permitted_roots(roots: Vec<PathBuf>, home: &Path, full_disk_access: bool) -> Vec<PathBuf> {
    if full_disk_access {
      return roots;
    }
    roots.into_iter().filter(|root| !is_consent_guarded(root, home)).collect()
  }

  /// Locations that stay unscanned without Full Disk Access.
  fn protected_location_count(home: &Path) -> u32 {
    let mut count = 0u32;
    for relative in [
      "Library/Containers",
      "Library/Group Containers",
      "Library/Application Support",
    ] {
      if let Ok(entries) = fs::read_dir(home.join(relative)) {
        count = count.saturating_add(entries.count() as u32);
      }
    }
    count
  }

  #[tauri::command(async)]
  pub fn check_full_disk_access() -> Result<PermissionStatus, String> {
    let granted = has_full_disk_access();
    let count = home_dir().map(|home| protected_location_count(&home)).unwrap_or(0);
    Ok(PermissionStatus {
      full_disk_access: granted,
      protected_location_count: count,
      message: if granted {
        "Full Disk Access granted. Xclense can scan app containers and support data without prompting.".to_string()
      } else {
        format!(
          "Full Disk Access is not granted, so {} protected app locations are skipped. Grant it once to scan them without repeated macOS prompts.",
          count
        )
      },
    })
  }

  /// Opens System Settings directly on the Full Disk Access list.
  #[tauri::command(async)]
  pub fn open_full_disk_access_settings() -> Result<(), String> {
    Command::new("open")
      .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
      .status()
      .map_err(|error| format!("failed to open System Settings: {}", error))?;
    Ok(())
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

  /// Higher is safer to delete. Combines the risk level, whether the tool
  /// regenerates the data, and how recently the item was touched.
  fn safety_score_for(risk: &str, regenerates: bool, protected: bool, modified_epoch_ms: u128) -> u8 {
    if protected {
      return 0;
    }
    let mut score: i32 = match risk {
      "low" => 80,
      "medium" => 45,
      _ => 12,
    };
    if regenerates {
      score += 14;
    }
    let now = now_epoch_ms();
    if modified_epoch_ms > 0 && now > modified_epoch_ms {
      let age_days = (now - modified_epoch_ms) / (24 * 60 * 60 * 1000);
      if age_days < 1 {
        score -= 18;
      } else if age_days < 7 {
        score -= 8;
      } else if age_days > 180 {
        score += 6;
      }
    }
    score.clamp(0, 99) as u8
  }

  struct ScanCtx<'a> {
    app: Option<&'a tauri::AppHandle>,
    scan_id: String,
    items: Vec<StorageScanItem>,
    accepted: Vec<PathBuf>,
    scanned: u32,
    item_counter: u32,
    reclaimable_bytes: u64,
    stage_index: u32,
    stage_total: u32,
    stage_label: String,
    stage_category_id: String,
    last_emit_ms: u128,
  }

  impl<'a> ScanCtx<'a> {
    fn new(app: Option<&'a tauri::AppHandle>, stage_total: u32) -> Self {
      Self {
        app,
        scan_id: format!("scan-{}", now_epoch_ms()),
        items: Vec::new(),
        accepted: Vec::new(),
        scanned: 0,
        item_counter: 0,
        reclaimable_bytes: 0,
        stage_index: 0,
        stage_total,
        stage_label: "Preparing".to_string(),
        stage_category_id: String::new(),
        last_emit_ms: 0,
      }
    }

    fn emit(&mut self, phase: &str, current_path: Option<String>, message: String, force: bool) {
      let app = match self.app {
        Some(value) => value,
        None => return,
      };
      let now = now_epoch_ms();
      if !force && now.saturating_sub(self.last_emit_ms) < 60 {
        return;
      }
      self.last_emit_ms = now;
      let _ = app.emit(
        "storage-scan-progress",
        StorageScanProgressEvent {
          scan_id: self.scan_id.clone(),
          phase: phase.to_string(),
          category_id: if self.stage_category_id.is_empty() {
            None
          } else {
            Some(self.stage_category_id.clone())
          },
          category_label: Some(self.stage_label.clone()),
          current_path,
          completed_stages: self.stage_index,
          total_stages: self.stage_total,
          scanned_paths: self.scanned,
          items_found: self.items.len() as u32,
          reclaimable_bytes: self.reclaimable_bytes,
          message,
        },
      );
    }

    fn begin_stage(&mut self, category: &StorageCategory) {
      self.stage_index = self.stage_index.saturating_add(1);
      self.stage_label = category.label.clone();
      self.stage_category_id = category.id.clone();
      let message = format!("Scanning {}…", category.label);
      self.emit("category_started", None, message, true);
    }

    fn note_path(&mut self, path: &Path) {
      self.scanned = self.scanned.saturating_add(1);
      let text = path.to_string_lossy().to_string();
      self.emit("path", Some(text), format!("Reading {}", self.stage_label), false);
    }

    /// Skips paths that overlap an item already recorded, so nested folders are
    /// never counted twice in the reclaimable total.
    fn overlaps_accepted(&self, path: &Path) -> bool {
      self
        .accepted
        .iter()
        .any(|existing| path.starts_with(existing) || existing.starts_with(path))
    }

    fn push_item(
      &mut self,
      category: &StorageCategory,
      path: &Path,
      size_bytes: u64,
      is_dir: bool,
      fallback_recommendation: &str,
    ) {
      if self.overlaps_accepted(path) {
        return;
      }

      let name = path
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());
      let hidden = name.starts_with('.');
      let known = known_entry_profile(&name);
      let base = known.clone().unwrap_or_else(|| category_default_profile(&category.id));

      let mut risk = base.risk.to_string();
      // Unknown hidden files in the home folder are configuration far more
      // often than they are junk, so they are treated conservatively.
      let unknown_home_dotfile = hidden && !is_dir && category.id == "hidden_home";
      if known.is_none() && (unknown_home_dotfile || category.risk_level == "high") {
        risk = "high".to_string();
      }

      let protected_reason = protected_path_reason(path);
      let protected = protected_reason.is_some();
      if protected {
        risk = "high".to_string();
      }

      let modified_epoch_ms = dir_modified_epoch_ms(path);
      let recommendation = match &protected_reason {
        Some(reason) => format!("{} {}", reason, base.recommendation),
        None => {
          if known.is_some() {
            base.recommendation.to_string()
          } else {
            fallback_recommendation.to_string()
          }
        }
      };

      self.item_counter = self.item_counter.saturating_add(1);
      self.accepted.push(path.to_path_buf());
      // Only items the user can actually select count as reclaimable.
      if !protected && risk != "high" {
        self.reclaimable_bytes = self.reclaimable_bytes.saturating_add(size_bytes);
      }

      let item = StorageScanItem {
        id: format!("scan-{}-{}", category.id, self.item_counter),
        category_id: category.id.clone(),
        path: path.to_string_lossy().to_string(),
        label: base.label.to_string(),
        owner: base.owner.to_string(),
        entry_kind: if is_dir { "directory".to_string() } else { "file".to_string() },
        hidden,
        identified: known.is_some(),
        protected,
        regenerates: base.regenerates && !protected,
        size_bytes,
        modified_epoch_ms,
        last_accessed_epoch_ms: dir_accessed_epoch_ms(path),
        risk_level: risk.clone(),
        safety_score: safety_score_for(&risk, base.regenerates, protected, modified_epoch_ms),
        impact_if_removed: base.impact.to_string(),
        recommendation,
      };

      let found_path = item.path.clone();
      let found_label = item.label.clone();
      self.items.push(item);
      self.emit(
        "item_found",
        Some(found_path),
        format!("Found {} ({})", found_label, format_size_short(size_bytes)),
        true,
      );
    }
  }

  fn format_size_short(bytes: u64) -> String {
    let value = bytes as f64;
    if value >= 1024.0 * 1024.0 * 1024.0 {
      format!("{:.1} GB", value / (1024.0 * 1024.0 * 1024.0))
    } else if value >= 1024.0 * 1024.0 {
      format!("{:.0} MB", value / (1024.0 * 1024.0))
    } else {
      format!("{} KB", bytes / 1024)
    }
  }

  fn scan_direct_children(
    ctx: &mut ScanCtx<'_>,
    category: &StorageCategory,
    root: &Path,
    min_size_bytes: u64,
    recommendation: &str,
  ) {
    let entries = match fs::read_dir(root) {
      Ok(value) => value,
      Err(_) => return,
    };

    for entry in entries.flatten() {
      let path = entry.path();
      if !path.exists() {
        continue;
      }
      let file_type = match entry.file_type() {
        Ok(value) => value,
        Err(_) => continue,
      };
      if file_type.is_symlink() {
        continue;
      }
      if ctx.overlaps_accepted(&path) {
        continue;
      }
      ctx.note_path(&path);
      let is_dir = file_type.is_dir();
      let size_bytes = entry_size_bytes(&path, is_dir);
      if size_bytes < min_size_bytes {
        continue;
      }
      ctx.push_item(category, &path, size_bytes, is_dir, recommendation);
    }
  }

  fn scan_single_path(
    ctx: &mut ScanCtx<'_>,
    category: &StorageCategory,
    path: &Path,
    min_size_bytes: u64,
    recommendation: &str,
  ) {
    if !path.exists() || ctx.overlaps_accepted(path) {
      return;
    }
    ctx.note_path(path);
    let is_dir = path.is_dir();
    let size_bytes = entry_size_bytes(path, is_dir);
    if size_bytes < min_size_bytes {
      return;
    }
    ctx.push_item(category, path, size_bytes, is_dir, recommendation);
  }

  /// Every dot-entry directly under the home folder, regardless of size, so the
  /// user can audit what actually lives in their root.
  fn scan_hidden_home_items(ctx: &mut ScanCtx<'_>, category: &StorageCategory, home: &Path) {
    let entries = match fs::read_dir(home) {
      Ok(value) => value,
      Err(_) => return,
    };

    for entry in entries.flatten() {
      let name = entry.file_name().to_string_lossy().to_string();
      if !name.starts_with('.') || name == ".Trash" {
        continue;
      }
      let file_type = match entry.file_type() {
        Ok(value) => value,
        Err(_) => continue,
      };
      if file_type.is_symlink() {
        continue;
      }
      let path = entry.path();
      if ctx.overlaps_accepted(&path) {
        continue;
      }
      ctx.note_path(&path);
      let is_dir = file_type.is_dir();
      let size_bytes = entry_size_bytes(&path, is_dir);
      ctx.push_item(
        category,
        &path,
        size_bytes,
        is_dir,
        "Hidden item in your home folder. Check the impact note before removing it.",
      );
    }
  }

  /// Dot-folders hidden inside ~/Library and ~/Library/Application Support.
  fn scan_hidden_support_items(
    ctx: &mut ScanCtx<'_>,
    category: &StorageCategory,
    home: &Path,
    full_disk_access: bool,
  ) {
    let mut roots = vec![home.join("Library"), home.join("Library/Caches")];
    // These two are TCC-protected: reading them without Full Disk Access raises
    // one "access data from other apps" dialog per app.
    if full_disk_access {
      roots.push(home.join("Library/Application Support"));
      roots.push(home.join("Library/Containers"));
    }

    for root in roots {
      let entries = match fs::read_dir(&root) {
        Ok(value) => value,
        Err(_) => continue,
      };
      for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with('.') || name == ".DS_Store" || name == ".localized" {
          continue;
        }
        let file_type = match entry.file_type() {
          Ok(value) => value,
          Err(_) => continue,
        };
        if file_type.is_symlink() {
          continue;
        }
        let path = entry.path();
        if ctx.overlaps_accepted(&path) {
          continue;
        }
        ctx.note_path(&path);
        let is_dir = file_type.is_dir();
        let size_bytes = entry_size_bytes(&path, is_dir);
        if size_bytes < 1024 * 1024 {
          continue;
        }
        ctx.push_item(
          category,
          &path,
          size_bytes,
          is_dir,
          "Hidden support folder. Identify the owning app before removing it.",
        );
      }
    }
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

  /// Walks project roots once, collecting node_modules folders and hidden build
  /// caches in the same pass.
  fn walk_project_dirs(
    ctx: &mut ScanCtx<'_>,
    root: &Path,
    node_modules_category: Option<&StorageCategory>,
    build_cache_category: Option<&StorageCategory>,
    max_depth: usize,
    max_scanned: u32,
  ) {
    if max_depth == 0 || ctx.scanned >= max_scanned || !root.is_dir() {
      return;
    }

    let entries = match fs::read_dir(root) {
      Ok(value) => value,
      Err(_) => return,
    };

    for entry in entries.flatten() {
      if ctx.scanned >= max_scanned {
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
      ctx.note_path(&path);
      let name = entry.file_name().to_string_lossy().to_string();

      if name == "node_modules" {
        if let Some(category) = node_modules_category {
          if ctx.overlaps_accepted(&path) {
            continue;
          }
          let size_bytes = directory_size_bytes(&path);
          if size_bytes >= 20 * 1024 * 1024 {
            ctx.push_item(
              category,
              &path,
              size_bytes,
              true,
              "Review this project dependency folder. Delete only if you can reinstall with npm, Yarn, or pnpm.",
            );
          }
        }
        continue;
      }

      if build_cache_dir_name(&name) {
        if let Some(category) = build_cache_category {
          if ctx.overlaps_accepted(&path) {
            continue;
          }
          let size_bytes = directory_size_bytes(&path);
          if size_bytes >= 10 * 1024 * 1024 {
            ctx.push_item(
              category,
              &path,
              size_bytes,
              true,
              "Build output folder. The toolchain recreates it on the next build.",
            );
          }
        }
        continue;
      }

      if should_skip_deep_scan_dir(&name) || name.ends_with(".app") || name.ends_with(".framework") {
        continue;
      }

      walk_project_dirs(
        ctx,
        &path,
        node_modules_category,
        build_cache_category,
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
    ctx: &mut ScanCtx<'_>,
    root: &Path,
    category: &StorageCategory,
    max_depth: usize,
    max_scanned: u32,
  ) {
    if max_depth == 0 || ctx.scanned >= max_scanned || !root.is_dir() {
      return;
    }

    let entries = match fs::read_dir(root) {
      Ok(value) => value,
      Err(_) => return,
    };

    for entry in entries.flatten() {
      if ctx.scanned >= max_scanned {
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
      ctx.note_path(&path);
      if file_type.is_file() {
        let size_bytes = fs::metadata(&path).map(|metadata| metadata.len()).unwrap_or(0);
        if size_bytes >= 500 * 1024 * 1024 && is_large_file_candidate(&path) {
          ctx.push_item(
            category,
            &path,
            size_bytes,
            false,
            "Large archive, installer, or media file. Review before sending it to Trash.",
          );
        }
      } else if file_type.is_dir() {
        let name = entry.file_name().to_string_lossy().to_string();
        if should_skip_deep_scan_dir(&name) || name.ends_with(".app") || name.ends_with(".photoslibrary") {
          continue;
        }
        walk_for_large_files(ctx, &path, category, max_depth - 1, max_scanned);
      }
    }
  }

  fn scan_app_container_caches(ctx: &mut ScanCtx<'_>, category: &StorageCategory, home: &Path) {
    let roots = [home.join("Library/Containers"), home.join("Library/Group Containers")];
    for root in roots {
      if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
          let cache_path = entry.path().join("Data/Library/Caches");
          if !cache_path.exists() {
            continue;
          }
          if ctx.overlaps_accepted(&cache_path) {
            continue;
          }
          ctx.note_path(&cache_path);
          let size_bytes = directory_size_bytes(&cache_path);
          if size_bytes < 10 * 1024 * 1024 {
            continue;
          }
          ctx.push_item(
            category,
            &cache_path,
            size_bytes,
            true,
            "App container cache. Safe to clean after quitting the related app; macOS/app can regenerate it.",
          );
        }
      }
    }
  }

  pub(crate) fn scan_storage_categories(app: Option<&tauri::AppHandle>) -> StorageScanResult {
    let started_at = now_epoch_ms();
    let categories = default_storage_categories();
    let mut ctx = ScanCtx::new(app, categories.len() as u32);
    ctx.emit(
      "started",
      None,
      "Starting deep scan of storage locations…".to_string(),
      true,
    );

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
          hidden_item_count: 0,
          protected_item_count: 0,
          full_disk_access: false,
          skipped_categories: Vec::new(),
        };
      }
    };

    // Without Full Disk Access, macOS raises a separate consent dialog for every
    // app container we touch. Skipping those categories entirely is the only way
    // to avoid burying the user in prompts mid-scan.
    let full_disk_access = has_full_disk_access();
    let mut skipped_categories: Vec<String> = Vec::new();

    for category in &categories {
      ctx.begin_stage(category);

      match category.id.as_str() {
        "hidden_home" => {
          scan_hidden_home_items(&mut ctx, category, &home);
          continue;
        }
        "hidden_support" => {
          scan_hidden_support_items(&mut ctx, category, &home, full_disk_access);
          continue;
        }
        // Every one of these reads a consent-guarded personal folder. Without
        // Full Disk Access each would raise its own dialog mid-scan, so the
        // whole category is skipped and reported instead.
        "app_container_caches" | "app_support_data" | "downloads" if !full_disk_access => {
          skipped_categories.push(category.id.clone());
          continue;
        }
        "app_container_caches" => {
          scan_app_container_caches(&mut ctx, category, &home);
          continue;
        }
        "node_modules" | "project_build_caches" => {
          // Both are collected in a single project walk, driven by node_modules.
          if category.id == "node_modules" {
            let build_cache_category = category_by_id(&categories, "project_build_caches");
            let roots = permitted_roots(
              vec![
                home.join("Developer"),
                home.join("Projects"),
                home.join("Sites"),
                home.join("Documents"),
                home.join("Desktop"),
                home.join("Downloads"),
                home.join("repos"),
                home.join("code"),
                home.join("work"),
              ],
              &home,
              full_disk_access,
            );
            for root in roots {
              walk_project_dirs(
                &mut ctx,
                &root,
                Some(category),
                build_cache_category.as_ref(),
                8,
                120_000,
              );
            }
          }
          continue;
        }
        "large_files" => {
          let roots = permitted_roots(
            vec![
              home.join("Downloads"),
              home.join("Desktop"),
              home.join("Documents"),
              home.join("Movies"),
              home.join("Pictures"),
              home.join("Developer"),
              home.join("Projects"),
              home.join("Sites"),
            ],
            &home,
            full_disk_access,
          );
          for root in roots {
            walk_for_large_files(&mut ctx, &root, category, 7, 120_000);
          }
          continue;
        }
        _ => {}
      }

      for prefix in &category.path_prefixes {
        let candidate = resolve_scan_prefix(&home, prefix);
        if !candidate.exists() {
          continue;
        }

        // The single chokepoint. Categories declare paths freely, and several
        // of them point inside TCC-guarded roots — reading one raises a dialog
        // mid-scan. Checking here means no category can reintroduce a prompt.
        if !full_disk_access && is_consent_guarded(&candidate, &home) {
          if !skipped_categories.contains(&category.id) {
            skipped_categories.push(category.id.clone());
          }
          continue;
        }

        match category.id.as_str() {
          "downloads" => scan_direct_children(
            &mut ctx,
            category,
            &candidate,
            20 * 1024 * 1024,
            "Review this download before cleanup. It may be an installer, archive, or user-created file.",
          ),
          "trash" => scan_direct_children(
            &mut ctx,
            category,
            &candidate,
            5 * 1024 * 1024,
            "Already in Trash. Send it through cleanup when you are sure it is no longer needed.",
          ),
          "user_caches" => scan_direct_children(
            &mut ctx,
            category,
            &candidate,
            10 * 1024 * 1024,
            "App cache folder. Safe to clean after quitting the related app; it can be regenerated.",
          ),
          "user_logs" => scan_direct_children(
            &mut ctx,
            category,
            &candidate,
            5 * 1024 * 1024,
            "Old diagnostic logs. Safe to clean unless you need them for troubleshooting.",
          ),
          "browser_cache" => scan_single_path(
            &mut ctx,
            category,
            &candidate,
            10 * 1024 * 1024,
            "Browser cache. Quit the browser first; it can rebuild these files automatically.",
          ),
          "developer_artifacts" => scan_direct_children(
            &mut ctx,
            category,
            &candidate,
            20 * 1024 * 1024,
            "Developer build/simulator artifact. Review active projects first; most of this data can be regenerated.",
          ),
          "package_manager_caches" => scan_single_path(
            &mut ctx,
            category,
            &candidate,
            20 * 1024 * 1024,
            "Package manager cache. Usually safe to clean; packages can be downloaded again later.",
          ),
          "app_support_data" => scan_direct_children(
            &mut ctx,
            category,
            &candidate,
            100 * 1024 * 1024,
            "App data folder — not a cache. Removing it resets the app's local state.",
          ),
          "system_temp" => scan_direct_children(
            &mut ctx,
            category,
            &candidate,
            20 * 1024 * 1024,
            "Temporary system item. Review carefully and avoid cleaning files from currently running apps.",
          ),
          _ => scan_single_path(
            &mut ctx,
            category,
            &candidate,
            10 * 1024 * 1024,
            "Review this reclaimable storage item before cleanup.",
          ),
        }
      }
    }

    let scanned_paths = ctx.scanned;
    let item_count = ctx.items.len();

    ctx.stage_index = ctx.stage_total;
    ctx.stage_label = "Complete".to_string();
    ctx.stage_category_id = String::new();
    ctx.emit(
      "completed",
      None,
      format!(
        "Scan complete — {} item(s) across {} location(s).",
        item_count, scanned_paths
      ),
      true,
    );

    let mut items = std::mem::take(&mut ctx.items);
    // Sort largest first.
    items.sort_by_key(|item| std::cmp::Reverse(item.size_bytes));
    let total_recoverable_bytes: u64 = items
      .iter()
      .filter(|item| !item.protected && item.risk_level != "high")
      .map(|item| item.size_bytes)
      .sum();
    let hidden_item_count = items.iter().filter(|item| item.hidden).count() as u32;
    let protected_item_count = items.iter().filter(|item| item.protected).count() as u32;

    StorageScanResult {
      started_at_epoch_ms: started_at,
      completed_at_epoch_ms: now_epoch_ms(),
      scanned_paths,
      items,
      categories,
      total_recoverable_bytes,
      hidden_item_count,
      protected_item_count,
      full_disk_access,
      skipped_categories,
    }
  }

  /// Runs off the main thread so progress events reach the UI while the scan
  /// is still walking the filesystem.
  #[tauri::command(async)]
  pub fn scan_storage_for_cleanup(app: tauri::AppHandle) -> Result<StorageScanResult, String> {
    Ok(scan_storage_categories(Some(&app)))
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

  /// Runs off the main thread so per-item progress events are delivered live.
  #[tauri::command(async)]
  pub fn cleanup_storage_items(
    app: tauri::AppHandle,
    request: CleanupRequest,
  ) -> Result<CleanupResult, String> {
    if request.item_ids.is_empty() {
      return Err("no items were requested for cleanup".to_string());
    }

    let scan = scan_storage_categories(None);
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
      if let Some(reason) = protected_path_reason(&path) {
        all_succeeded = false;
        let message = format!("Skipped protected item. Nothing was moved or deleted. {}", reason);
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
    .plugin(tauri_plugin_updater::Builder::new().build())
    .plugin(tauri_plugin_process::init())
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
      commands::cleanup_storage_items,
      commands::check_full_disk_access,
      commands::diagnose_memory_condition,
      commands::open_full_disk_access_settings
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}


#[cfg(test)]
mod permission_gate_tests {
  /// A scan must raise **zero** consent dialogs when Full Disk Access is absent.
  ///
  /// This has regressed twice. macOS guards `~/Desktop`, `~/Documents`,
  /// `~/Downloads`, `~/Movies` and `~/Pictures` with per-folder prompts, and
  /// `~/Library/Containers` with a prompt *per container* — roughly a thousand
  /// on a normal machine. Both times the gate covered some of those paths and
  /// silently missed others, which is invisible in code review and obvious to
  /// the user as a wall of dialogs.
  ///
  /// Asserting on paths rather than on the gate's own bookkeeping is
  /// deliberate: it fails when a *new* walk root is added without being
  /// filtered, which is exactly how this broke before.
  ///
  /// Skips itself when FDA is held, since the gate is then a no-op by design.
  #[test]
  fn scan_touches_no_consent_guarded_path_without_full_disk_access() {
    let home = std::path::PathBuf::from(std::env::var("HOME").expect("HOME must be set"));
    // The full guarded set, not just the personal folders. An earlier version
    // of this test checked only the latter and passed while `browser_cache`
    // read `Library/Containers/com.apple.Safari` on every scan.
    let guarded = crate::commands::CONSENT_GUARDED_ROOTS;

    let result = crate::commands::scan_storage_categories(None);

    if result.full_disk_access {
      eprintln!("Full Disk Access is held — the gate is inert, nothing to verify.");
      return;
    }

    let offenders: Vec<&str> = result
      .items
      .iter()
      .map(|item| item.path.as_str())
      .filter(|path| {
        guarded.iter().any(|dir| {
          home
            .join(dir)
            .to_str()
            .is_some_and(|guarded_root| path.starts_with(guarded_root))
        })
      })
      .collect();

    assert!(
      offenders.is_empty(),
      "scan read {} consent-guarded path(s) without Full Disk Access, \
       which raises a dialog for each one. First few: {:?}",
      offenders.len(),
      offenders.iter().take(5).collect::<Vec<_>>()
    );

    for required in ["downloads", "app_container_caches", "app_support_data"] {
      assert!(
        result.skipped_categories.iter().any(|id| id == required),
        "category '{required}' reads guarded paths and must be reported as skipped, \
         so the UI can tell the user what was left out"
      );
    }
  }
}



#[cfg(test)]
mod memory_diagnosis_tests {
  use crate::commands::diagnose_memory;
  use crate::{MemoryFailureMode, ProcessInfo, SystemHealth};

  const GB: u64 = 1024 * 1024 * 1024;

  fn health(wired: u64, active: u64, inactive: u64, compressed: u64, swap_used: u64, swap_total: u64) -> SystemHealth {
    let total = 8 * GB;
    let committed = wired + active + compressed;
    SystemHealth {
      memory_total_bytes: total,
      memory_free_bytes: total.saturating_sub(committed + inactive),
      memory_used_bytes: committed + inactive,
      memory_pressure_percent: (committed as f64 / total as f64) * 100.0,
      memory_wired_bytes: wired,
      memory_active_bytes: active,
      memory_inactive_bytes: inactive,
      memory_compressed_bytes: compressed,
      swap_total_bytes: swap_total,
      swap_used_bytes: swap_used,
      swap_free_bytes: swap_total.saturating_sub(swap_used),
      swap_used_percent: if swap_total == 0 { 0.0 } else { (swap_used as f64 / swap_total as f64) * 100.0 },
      swapouts: 0,
      load_average_1m: 1.0,
      load_average_5m: 1.0,
      load_average_15m: 1.0,
      scanned_at_epoch_ms: 0,
    }
  }

  fn process(name: &str, memory_percent: f32) -> ProcessInfo {
    ProcessInfo { pid: 1, name: name.to_string(), cpu_percent: 1.0, memory_percent, state: "running".to_string() }
  }

  /// The condition that motivated this work: 8 GB machine, swap at 97%.
  /// The previous logic reported 99% pressure and recommended `purge`, which
  /// can only touch the 0.58 GB of inactive memory and leaves swap untouched.
  #[test]
  fn swap_exhaustion_demands_a_restart_not_a_cache_purge() {
    let d = diagnose_memory(&health(4 * GB, GB, GB / 2, 2 * GB, 14 * GB, 15 * GB), &[]);
    assert_eq!(d.mode, MemoryFailureMode::SwapThrashing);
    assert!(d.restart_required);
    assert_eq!(d.suggested_action, "restart_to_reclaim_memory");
    // Claiming reclaimable bytes here would be the same lie as before.
    assert_eq!(d.reclaimable_bytes, 0, "nothing can be freed while swap is the problem");
  }

  /// Wired memory is unpageable, so no amount of quitting apps recovers it.
  #[test]
  fn wired_bloat_demands_a_restart() {
    let d = diagnose_memory(&health(4 * GB, GB, GB / 2, GB / 2, 0, 15 * GB), &[]);
    assert_eq!(d.mode, MemoryFailureMode::WiredBloat);
    assert!(d.restart_required);
    assert_eq!(d.reclaimable_bytes, 0);
  }

  /// Swap must outrank wired: when both are bad, paging dominates and is the
  /// thing the user is actually feeling.
  #[test]
  fn swap_outranks_wired_when_both_are_critical() {
    let d = diagnose_memory(&health(4 * GB, GB, GB / 2, 2 * GB, 14 * GB, 15 * GB), &[]);
    assert_eq!(d.mode, MemoryFailureMode::SwapThrashing);
  }

  /// The one case where purge is honest.
  #[test]
  fn large_cache_is_the_only_case_that_recommends_purge() {
    let d = diagnose_memory(&health(GB, GB, 3 * GB, GB / 2, GB, 15 * GB), &[]);
    assert_eq!(d.mode, MemoryFailureMode::CachePressure);
    assert_eq!(d.suggested_action, "free_inactive_memory");
    assert_eq!(d.reclaimable_bytes, 3 * GB);
    assert!(!d.restart_required);
  }

  #[test]
  fn a_dominant_process_is_named_rather_than_blamed_on_the_system() {
    let d = diagnose_memory(
      &health(GB, 4 * GB, GB / 2, GB / 2, GB, 15 * GB),
      &[process("Xcode", 40.0), process("Finder", 1.0)],
    );
    assert_eq!(d.mode, MemoryFailureMode::ProcessHog);
    assert!(d.headline.contains("Xcode"));
    assert!(d.reclaimable_bytes > 0);
  }

  /// Many mid-sized processes: no hog to blame and nothing meaningful to free.
  #[test]
  fn many_small_processes_are_reported_as_having_no_single_cause() {
    let processes: Vec<ProcessInfo> = (0..8).map(|i| process(&format!("app{i}"), 5.0)).collect();
    let d = diagnose_memory(&health(2 * GB, 4 * GB, GB / 2, GB, GB, 15 * GB), &processes);
    assert_eq!(d.mode, MemoryFailureMode::DeathByAThousandCuts);
    assert!(!d.restart_required);
  }

  /// A machine with a warm file cache is healthy. The old `total - free`
  /// formula reported this as ~95% and indistinguishable from a dying system.
  #[test]
  fn a_warm_cache_is_not_reported_as_pressure() {
    let d = diagnose_memory(&health(GB, GB, 4 * GB, GB / 2, 0, 15 * GB), &[]);
    assert!(d.mode != MemoryFailureMode::SwapThrashing);
    assert!(d.mode != MemoryFailureMode::WiredBloat);
    assert!(!d.restart_required);
  }
}
