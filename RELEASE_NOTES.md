# v0.2.0-alpha.8 — Alpha Release

## 🔄 What's Changed (v0.2.0-alpha.7 ➔ v0.2.0-alpha.8)

- **Channel**: Alpha Release (Preview Channel)
- **Platform**: macOS 11+ · Apple Silicon and Intel (universal binary)
- **Install**: Already on `v0.2.0-alpha.2` or later? Do nothing — this installs itself.

> Xclense now works out **why** memory is under pressure, and says so — including when
> the honest answer is that nothing it can run will help.

### ✨ New Features & Enhancements

- **Swap is measured.** It never was. A machine paging itself to death looked identical
  to one that had never swapped, because the only value read from `vm_stat` was
  `Pages free`. Swap usage, wired, active, inactive and compressed memory are all
  collected now.

- **Memory diagnosis.** Xclense distinguishes five conditions that were previously
  indistinguishable and each need a different answer:

  | Condition | What actually helps |
  | --- | --- |
  | Swap near capacity | Restart — macOS drains swap only as demand falls |
  | Wired memory bloated | Restart — wired memory is never released to apps |
  | Large file cache | Releasing it genuinely frees that much |
  | One dominant process | Quit that process, named in the report |
  | No single cause | Close several, or restart |

- **Advice you can act on.** Where a remedy exists, Xclense says what it will recover.
  Where none does, it says a restart is required rather than running something that
  reports success and changes nothing.

### 🐛 Fixed Bugs & Issues

- **Memory pressure was misleading in both directions.** It was computed as
  `(total − free) / total`, which counts reclaimable file cache as used. An idle Mac
  with a warm cache reported ~95% and looked identical to one that was genuinely dying.
  Pressure is now wired + active + compressed — only memory that cannot be reclaimed on
  demand.

- **"Free inactive memory" no longer claims a fix it did not deliver.** It ran
  `purge` regardless of whether there was anything worth releasing, and reported
  success either way. It now skips with an explanation when cached memory is under 1 GB,
  and is never suggested at all while swap or wired memory is the real problem —
  `purge` cannot touch either.

### 🧪 Quality

- Seven tests covering every failure mode, including that swap outranks wired memory
  when both are critical, and that a warm file cache is **not** reported as pressure —
  the false positive the old formula produced on healthy machines.

### 📊 Validated against a real machine

Developed against an 8 GB Mac in genuine distress, not a synthetic fixture:

```text
swap 14.53 / 15.00 GB (97%) · wired 4.32 GB (54%) · inactive 0.58 GB

mode:     SwapThrashing
action:   restart required
reclaim:  0.00 GB
```

The previous build would have reported 99% pressure, run `purge`, declared success, and
left the machine at 97% swap.

### ⚠️ Alpha Channel Notes

- The diagnosis thresholds (35% wired, 80% swap, 1 GB cache floor) are reasoned rather
  than calibrated across many machines. If a verdict looks wrong on your hardware,
  please report it with the figures shown on the health card — that is exactly the
  feedback needed.
- Full Disk Access may need re-granting after an update; an unsigned app's code identity
  changes with every build.
- Cleanup always routes through Finder's Trash — Xclense never deletes permanently.

### 🔓 First launch on macOS 15 Sequoia and macOS 26 Tahoe

This build is **unsigned and un-notarized**, so macOS blocks the first launch with:

> **"Xclense" Not Opened** — Apple could not verify "Xclense" is free of malware…
> **[ Done ] [ Move to Bin ]**

The app is fine; it just has no Apple Developer ID. **Right-click ➔ Open no longer
works** — Apple removed that bypass in Sequoia. Do one of these instead:

1. Click **Done**, then open **System Settings ➔ Privacy & Security**, scroll to
   Security, and click **Open Anyway** next to the Xclense message. Authenticate, launch
   again, confirm. (The message expires about an hour after the blocked launch.)
2. Or, in Terminal: `xattr -dr com.apple.quarantine /Applications/Xclense.app`

This only affects DMG installs. **In-app OTA updates are never blocked**, because the
updater downloads them directly rather than through a browser.

Report issues at <https://github.com/sudsarkar13/xclense/issues> with the version
shown in the sidebar.
