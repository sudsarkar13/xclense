#!/bin/bash
# ==============================================================================
#  mac_disk_scanner.sh  v2.1.0
#  Scans all hard disk / SSD storage on any MacBook (all macOS versions)
#  Compatible with: PowerPC, Intel (32/64-bit), Apple Silicon (M-series)
#  macOS versions: 10.4 Tiger → macOS 26+ (and beyond)
#
#  FIXES in v2.1.0:
#    • Colours now always render correctly in Terminal
#    • Summary no longer double-counts APFS volumes (uses only physical disk size)
#    • 'map' / auto-mount pseudo-volumes are fully excluded
#    • Mount-point parser hardened against paths containing spaces or % signs
#
#  Usage:
#    chmod +x mac_disk_scanner.sh   # Make executable (first time only)
#    ./mac_disk_scanner.sh          # Run the script
#    ./mac_disk_scanner.sh --json   # Output in JSON format
#    ./mac_disk_scanner.sh --help   # Show usage help
# ==============================================================================

SCRIPT_VERSION="2.1.0"

# ── Force colour escape codes using $'...' ANSI literals ──────────────────────
BOLD=$'\033[1m';   RESET=$'\033[0m'
RED=$'\033[0;31m'; GREEN=$'\033[0;32m'; YELLOW=$'\033[0;33m'
CYAN=$'\033[0;36m'; BLUE=$'\033[0;34m'; MAGENTA=$'\033[0;35m'
WHITE=$'\033[0;37m'

# ── Helpers ────────────────────────────────────────────────────────────────────
hr() { printf '%0.s─' {1..70}; printf '\n'; }

show_help() {
  cat <<EOF
mac_disk_scanner.sh v${SCRIPT_VERSION}
Scans all physical disks on any MacBook and reports storage details.

Usage:
  ./mac_disk_scanner.sh           Standard coloured report
  ./mac_disk_scanner.sh --json    Machine-readable JSON output
  ./mac_disk_scanner.sh --help    Show this help message

Output:
  • Mac model, chip, macOS version
  • Physical disk: type (NVMe/SSD/HDD/Fusion), TRUE total capacity
  • Per-volume: mount point, filesystem, total / used / free, % bar
  • Summary based on PHYSICAL disk size (not inflated by APFS sharing)
  • Storage health tips + APFS snapshot warning
EOF
  exit 0
}

# ── Argument parsing ───────────────────────────────────────────────────────────
JSON_MODE=false
for arg in "$@"; do
  case "$arg" in
    --json)    JSON_MODE=true ;;
    --help|-h) show_help ;;
    *) printf "Unknown argument: %s  (use --help)\n" "$arg"; exit 1 ;;
  esac
done

# ── System information ─────────────────────────────────────────────────────────
get_macos_version() { sw_vers -productVersion 2>/dev/null || uname -r; }

get_architecture() {
  case "$(uname -m 2>/dev/null)" in
    arm64)  echo "Apple Silicon (arm64)" ;;
    x86_64) echo "Intel 64-bit (x86_64)" ;;
    i386)   echo "Intel 32-bit (i386)" ;;
    ppc*)   echo "PowerPC ($(uname -m))" ;;
    *)      echo "$(uname -m)" ;;
  esac
}

get_mac_model() {
  local m
  m=$(system_profiler SPHardwareDataType 2>/dev/null \
      | awk -F': ' '/Model (Name|Identifier)/{print $2; exit}')
  [ -z "$m" ] && m=$(sysctl -n hw.model 2>/dev/null)
  [ -z "$m" ] && m="Unknown Mac Model"
  echo "$m"
}

# ── Disk type detection ────────────────────────────────────────────────────────
get_disk_type() {
  local disk="$1" info dtype="Unknown"
  command -v diskutil &>/dev/null || { echo "$dtype"; return; }
  info=$(diskutil info "$disk" 2>/dev/null)
  echo "$info" | grep -qi "NVMe"                                     && dtype="NVMe SSD"
  echo "$info" | grep -qi "APPLE SSD\|Apple Fabric\|Apple Internal"  && dtype="Apple Internal SSD"
  echo "$info" | grep -qi "Fusion"                                   && dtype="Fusion Drive"
  echo "$info" | grep -qi "Solid State: Yes\|SSD: Yes" && [ "$dtype" = "Unknown" ] && dtype="SSD"
  echo "$info" | grep -qi "Rotational Rate"             && [ "$dtype" = "Unknown" ] && dtype="HDD (Rotational)"
  echo "$dtype"
}

# ── Physical disk size in bytes ────────────────────────────────────────────────
get_disk_size_bytes() {
  local disk="$1" size=""
  command -v diskutil &>/dev/null || { echo 0; return; }
  # "Disk Size: 500.1 GB (500107862016 Bytes)" — grab the raw byte count
  size=$(diskutil info "$disk" 2>/dev/null \
         | grep -Eo '\([0-9]+ Bytes\)' | tr -d '() Bytes')
  echo "${size:-0}"
}

# ── Human-readable bytes ───────────────────────────────────────────────────────
bytes_to_human() {
  local b="${1:-0}"
  [ "$b" -eq 0 ] 2>/dev/null && { echo "0 B"; return; }
  if   [ "$b" -ge 1099511627776 ]; then printf "%.2f TB\n" "$(echo "scale=4;$b/1099511627776"|bc)"
  elif [ "$b" -ge 1073741824 ];    then printf "%.2f GB\n" "$(echo "scale=4;$b/1073741824"|bc)"
  elif [ "$b" -ge 1048576 ];       then printf "%.2f MB\n" "$(echo "scale=4;$b/1048576"|bc)"
  elif [ "$b" -ge 1024 ];          then printf "%.2f KB\n" "$(echo "scale=4;$b/1024"|bc)"
  else echo "${b} B"
  fi
}

# ── ASCII usage bar ────────────────────────────────────────────────────────────
draw_bar() {
  local pct="${1:-0}" width=40
  local filled=$(( pct * width / 100 ))
  local empty=$(( width - filled ))
  local bar="" i=0
  while [ $i -lt $filled ]; do bar="${bar}█"; i=$((i+1)); done
  i=0
  while [ $i -lt $empty ];  do bar="${bar}░"; i=$((i+1)); done
  if   [ "$pct" -ge 90 ]; then printf "%s[%s] %d%%%s" "$RED"    "$bar" "$pct" "$RESET"
  elif [ "$pct" -ge 70 ]; then printf "%s[%s] %d%%%s" "$YELLOW" "$bar" "$pct" "$RESET"
  else                          printf "%s[%s] %d%%%s" "$GREEN"  "$bar" "$pct" "$RESET"
  fi
}

# ── Real volume test — excludes devfs, map auto-mounts, nullfs, etc. ──────────
is_real_volume() {
  local fs="$1"
  echo "$fs" | grep -qiE '^(devfs|map|none|tmpfs|nullfs|union|autofs|securityfs)' && return 1
  return 0
}

# ── List physical disk devices ─────────────────────────────────────────────────
get_physical_disks() {
  if command -v diskutil &>/dev/null; then
    diskutil list 2>/dev/null \
      | awk '/^\/dev\/disk[0-9]+[[:space:]]/{print $1}' \
      | sed 's|/dev/||'
  else
    ls /dev/disk[0-9] 2>/dev/null | sed 's|/dev/||' || true
  fi
}

# ── Parse df output safely (handles % in paths, spaces in mount names) ────────
# Output: tab-separated: fs  total_kb  used_kb  avail_kb  pct  mount
parse_df() {
  df -Pk 2>/dev/null | tail -n +2 | awk '
  {
    fs=$1; total=$2; used=$3; avail=$4; pct=$5
    gsub(/%/, "", pct)
    mount=""
    for (i=6; i<=NF; i++) mount = mount (i>6 ? " " : "") $i
    printf "%s\t%s\t%s\t%s\t%s\t%s\n", fs, total, used, avail, pct, mount
  }'
}

# ══════════════════════════════════════════════════════════════════════════════
#  STANDARD OUTPUT
# ══════════════════════════════════════════════════════════════════════════════
output_standard() {
  printf "\n%s" "${BOLD}${CYAN}"
  hr
  printf "   💾  MacBook Hard Disk / SSD Storage Scanner  v%s\n" "$SCRIPT_VERSION"
  hr
  printf "%s\n\n" "$RESET"

  # System info
  local mac_model arch macos_ver
  mac_model=$(get_mac_model)
  arch=$(get_architecture)
  macos_ver=$(get_macos_version)

  printf "%s▶ SYSTEM INFORMATION%s\n" "${BOLD}${BLUE}" "$RESET"
  hr
  printf "  %-24s %s%s%s\n" "Mac Model:"      "$MAGENTA" "$mac_model"                       "$RESET"
  printf "  %-24s %s%s%s\n" "Architecture:"   "$MAGENTA" "$arch"                            "$RESET"
  printf "  %-24s %s%s%s\n" "macOS Version:"  "$MAGENTA" "$macos_ver"                       "$RESET"
  printf "  %-24s %s%s%s\n" "Scan Date/Time:" "$MAGENTA" "$(date '+%Y-%m-%d %H:%M:%S %Z')" "$RESET"
  printf "\n"

  # Physical disks
  printf "%s▶ PHYSICAL DISKS%s\n" "${BOLD}${BLUE}" "$RESET"
  hr

  local grand_physical_bytes=0 disk_count=0 disks
  disks=$(get_physical_disks)

  if [ -z "$disks" ]; then
    printf "  %sCould not enumerate physical disks (diskutil unavailable).%s\n" "$YELLOW" "$RESET"
  else
    while IFS= read -r disk; do
      [ -z "$disk" ] && continue
      disk_count=$((disk_count + 1))
      local dtype size_bytes size_human
      dtype=$(get_disk_type "$disk")
      size_bytes=$(get_disk_size_bytes "$disk")
      size_human=$(bytes_to_human "$size_bytes")
      grand_physical_bytes=$((grand_physical_bytes + size_bytes))

      printf "\n  %s%sDisk %d — /dev/%s%s\n" "$BOLD" "$WHITE" "$disk_count" "$disk" "$RESET"
      printf "  %-26s %s%s%s\n" "  Storage Type:"  "$CYAN"  "$dtype"      "$RESET"
      printf "  %-26s %s%s%s\n" "  True Capacity:" "$GREEN" "$size_human" "$RESET"

      if command -v diskutil &>/dev/null; then
        printf "  %-26s\n" "  Partitions:"
        diskutil list "/dev/$disk" 2>/dev/null \
          | grep -E '^\s+[0-9]+:' \
          | sed 's/^/     /'
      fi
    done <<< "$disks"
  fi
  printf "\n"

  # Volume details
  printf "%s▶ VOLUME STORAGE DETAILS%s\n" "${BOLD}${BLUE}" "$RESET"
  hr

  local volume_count=0 summary_used_bytes=0

  while IFS=$'\t' read -r fs total_kb used_kb avail_kb pct mount; do
    is_real_volume "$fs" || continue
    [ "${total_kb:-0}" -eq 0 ] 2>/dev/null && continue

    volume_count=$((volume_count + 1))
    local total_b used_b avail_b
    total_b=$((total_kb * 1024))
    used_b=$((used_kb  * 1024))
    avail_b=$((avail_kb * 1024))

    printf "\n  %s%sVolume %d — %s%s\n" "$BOLD" "$WHITE" "$volume_count" "$mount" "$RESET"
    printf "  %-26s %s%s%s\n" "  Filesystem:"  "$CYAN"   "$fs"                    "$RESET"
    printf "  %-26s %s%s%s\n" "  Total Space:" "$GREEN"  "$(bytes_to_human $total_b)" "$RESET"
    printf "  %-26s %s%s%s\n" "  Used Space:"  "$RED"    "$(bytes_to_human $used_b)"  "$RESET"
    printf "  %-26s %s%s%s\n" "  Free Space:"  "$YELLOW" "$(bytes_to_human $avail_b)" "$RESET"
    printf "  %-26s "         "  Usage:"
    draw_bar "$pct"
    printf "\n"

    # Only sum 'used' bytes (per-volume used is real; total is shared in APFS)
    summary_used_bytes=$((summary_used_bytes + used_b))

  done < <(parse_df)
  printf "\n"

  # Summary — use PHYSICAL disk size as authoritative total
  printf "%s▶ SUMMARY (Physical Disk Basis)%s\n" "${BOLD}${BLUE}" "$RESET"
  hr

  local phy_free_bytes overall_pct
  phy_free_bytes=$((grand_physical_bytes - summary_used_bytes))
  [ "$phy_free_bytes" -lt 0 ] && phy_free_bytes=0
  overall_pct=0
  [ "$grand_physical_bytes" -gt 0 ] 2>/dev/null && \
    overall_pct=$((summary_used_bytes * 100 / grand_physical_bytes))

  printf "\n"
  printf "  %-28s %s%d%s\n" "  Logical Volumes Found:" "$WHITE"  "$volume_count"                       "$RESET"
  printf "  %-28s %s%s%s\n" "  Physical Disk Total:"   "$GREEN"  "$(bytes_to_human $grand_physical_bytes)" "$RESET"
  printf "  %-28s %s%s%s\n" "  Total Used:"            "$RED"    "$(bytes_to_human $summary_used_bytes)"   "$RESET"
  printf "  %-28s %s%s%s\n" "  Total Free:"            "$YELLOW" "$(bytes_to_human $phy_free_bytes)"       "$RESET"
  printf "  %-28s "         "  Overall Usage:"
  draw_bar "$overall_pct"
  printf "\n\n"

  # Health tips
  printf "%s▶ STORAGE HEALTH TIPS%s\n" "${BOLD}${BLUE}" "$RESET"
  hr

  if [ "$overall_pct" -ge 90 ]; then
    printf "  %s⚠  CRITICAL: Disk over 90%% full!%s\n" "$RED" "$RESET"
    printf "     → Move files to iCloud or external drive immediately.\n"
    printf "     → macOS may become unstable or fail to install updates.\n"
  elif [ "$overall_pct" -ge 70 ]; then
    printf "  %s⚡ WARNING: Disk over 70%% full.%s\n" "$YELLOW" "$RESET"
    printf "     → Clean Downloads, Trash, or large media files.\n"
    printf "     → Apple Menu → About This Mac → Storage → Manage\n"
  else
    printf "  %s✔  Storage is healthy (under 70%% used).%s\n" "$GREEN" "$RESET"
    printf "     → Keep monitoring periodically.\n"
  fi

  # Warn specifically about the Data volume (user files live here)
  local data_pct
  data_pct=$(parse_df | awk -F'\t' '$6=="/System/Volumes/Data"{print $5; exit}')
  if [ -n "$data_pct" ] && [ "$data_pct" -ge 85 ] 2>/dev/null; then
    printf "\n  %s⚠  /System/Volumes/Data is %d%% full (your personal files!).%s\n" \
           "$YELLOW" "$data_pct" "$RESET"
    printf "     → Move large files to iCloud Drive or an external disk.\n"
  fi

  # APFS local snapshots
  if command -v tmutil &>/dev/null; then
    local snaps
    snaps=$(tmutil listlocalsnapshots / 2>/dev/null | grep -c 'com.apple' || echo 0)
    if [ "${snaps:-0}" -gt 0 ]; then
      printf "\n  %sℹ  Time Machine local snapshots found: %d%s\n" "$CYAN" "$snaps" "$RESET"
      printf "     → Run: sudo tmutil deletelocalsnapshots /\n"
      printf "     → This can reclaim several GB of hidden snapshot space.\n"
    fi
  fi

  printf "\n"
  hr
  printf "  %sScan complete.%s  Use %s--json%s for machine-readable output.\n" \
         "$BOLD" "$RESET" "$CYAN" "$RESET"
  hr
  printf "\n"
}

# ══════════════════════════════════════════════════════════════════════════════
#  JSON OUTPUT
# ══════════════════════════════════════════════════════════════════════════════
output_json() {
  printf '{\n'
  printf '  "scanner_version": "%s",\n' "$SCRIPT_VERSION"
  printf '  "timestamp": "%s",\n'       "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  printf '  "system": {\n'
  printf '    "mac_model": "%s",\n'     "$(get_mac_model)"
  printf '    "architecture": "%s",\n'  "$(get_architecture)"
  printf '    "macos_version": "%s"\n'  "$(get_macos_version)"
  printf '  },\n'
  printf '  "physical_disks": [\n'
  local first=true
  while IFS= read -r disk; do
    [ -z "$disk" ] && continue
    $first || printf ',\n'; first=false
    printf '    {"device":"/dev/%s","type":"%s","size_bytes":%d}' \
           "$disk" "$(get_disk_type "$disk")" "$(get_disk_size_bytes "$disk")"
  done < <(get_physical_disks)
  printf '\n  ],\n'
  printf '  "volumes": [\n'
  first=true
  while IFS=$'\t' read -r fs total_kb used_kb avail_kb pct mount; do
    is_real_volume "$fs" || continue
    [ "${total_kb:-0}" -eq 0 ] 2>/dev/null && continue
    $first || printf ',\n'; first=false
    printf '    {"mount":"%s","filesystem":"%s","total_bytes":%d,"used_bytes":%d,"free_bytes":%d,"pct_used":%s}' \
           "$mount" "$fs" \
           "$((total_kb*1024))" "$((used_kb*1024))" "$((avail_kb*1024))" "$pct"
  done < <(parse_df)
  printf '\n  ]\n}\n'
}

# ── Entry point ────────────────────────────────────────────────────────────────
if $JSON_MODE; then output_json; else output_standard; fi