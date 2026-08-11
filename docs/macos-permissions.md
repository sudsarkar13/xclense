# macOS Permissions — Why Xclense Asks for Full Disk Access

## The problem this solves

macOS 14 (Sonoma) and later guard per-app data behind a TCC consent prompt:

> **"Xclense" would like to access data from other apps.**
> Keeping app data separate makes it easier to manage your privacy and security.

The prompt is raised **per app container**, not once per app. A storage scanner walking
`~/Library/Containers` hits every one of them. On a representative machine:

| Location | Entries |
| --- | --- |
| `~/Library/Containers` | 742 |
| `~/Library/Group Containers` | 149 |
| `~/Library/Application Support` | 104 |

That is a potential ~1,000 consent dialogs during a single scan, each blocking the scan
until answered. This was reported during the `v0.2.0-alpha.3` alpha and fixed in
`v0.2.0-alpha.4`.

## How Xclense behaves now

Xclense checks for **Full Disk Access** before scanning, and branches:

**Without Full Disk Access** — the TCC-protected categories are skipped entirely:

- `app_container_caches` (`~/Library/Containers`, `~/Library/Group Containers`)
- `app_support_data` (`~/Library/Application Support`)
- the Containers and Application Support roots of `hidden_support`

No protected path is touched, so **no prompt is ever raised**. The scan still covers
user caches, logs, Downloads, Trash, browser caches, developer artifacts, package
manager caches, hidden home items, project build caches, large files, and system temp
directories. In testing that is 72 of the 77 items a fully-permitted scan finds.

The Scan & clean dialog shows an amber notice with the number of skipped locations and
a button that opens System Settings at the right pane.

**With Full Disk Access** — everything is scanned, and macOS raises no prompts at all,
because FDA supersedes the per-app-data consent requirement.

## Granting it

1. Open **System Settings ➔ Privacy & Security ➔ Full Disk Access**
   (the in-app button deep-links there via
   `x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles`)
2. Enable **Xclense** in the list, adding it with **+** if it is not shown
3. macOS may require quitting and reopening Xclense
4. Click **"I've granted it — re-check"** in the dialog, then re-scan

## How detection works

```rust
fs::File::open(home.join("Library/Application Support/com.apple.TCC/TCC.db")).is_ok()
```

`TCC.db` is readable only by processes holding Full Disk Access, and attempting to open
it **does not itself raise a prompt** — it just fails. That makes it a safe probe. The
check runs at the start of every scan, so granting access mid-session is picked up on
the next scan without restarting the app.

## Why not just ask once up front?

There is no API to request Full Disk Access programmatically, and no single prompt that
covers all app containers. macOS only offers:

- the per-container consent prompt (what we are avoiding), or
- the user manually adding the app to the Full Disk Access list

So the honest options are "skip protected paths" or "prompt ~1,000 times". Xclense
skips, reports clearly what was skipped, and makes granting access one click away.

## Note for development builds

TCC identity is tied to the binary's signature and path. An unsigned local build gets a
new identity on every rebuild, so any grant is forgotten and prompts reappear. Testing
permission behaviour is more reliable against an installed `.app` from the DMG than
against `target/release`.
