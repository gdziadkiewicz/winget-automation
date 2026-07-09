# Copilot Instructions — winget-automation

## Project overview

**winget-automation** is a Windows system-tray application written in Rust.
It autostarts with Windows, periodically calls `winget upgrade` to detect outdated
packages, parses the tabular output, and surfaces a balloon (toast) notification
listing the packages that have newer versions available.

### Target platform
Windows 10/11 — `x86_64-pc-windows-msvc`.  
The binary is not useful on Linux/macOS, but most logic (parsing, data structures)
can be compiled and unit-tested on any platform.

---

## Repository layout

```
src/main.rs   — all current logic (winget invocation + parser + entry point)
Cargo.toml    — package manifest; add dependencies here
README.md     — sample winget output used as a reference for the parser
```

---

## Current state of the code (`src/main.rs`)

| Area | Status |
|---|---|
| `winget_update_raw()` | Calls `winget upgrade` and returns stdout as UTF-8 string |
| `parse_winget_raw()` | Parses the fixed-width table; detects column offsets from the header line |
| `WingetUpdateOutput` | Struct holding name / id / installed version / available version / source |
| `PackageSource` | Enum: `WinGet` \| `MsStore`; parsed from the "Source" column |
| System tray | **Not yet implemented** |
| Balloon notification | **Not yet implemented** |
| Periodic scheduling | **Not yet implemented** |
| Autostart | **Not yet implemented** |

---

## Key TODOs left in the code

- Replace `Box<dyn Error>` with a typed error hierarchy (use `thiserror` or a hand-rolled
  enum) so call sites can match on specific variants.
- Implement the Windows system-tray icon (consider the `tray-icon` or `winit` + `tray-item`
  crates, or raw `winapi`/`windows` crate calls).
- Implement balloon / toast notifications (`windows` crate `Windows::UI::Notifications` or
  the older `Shell_NotifyIcon` API with `NIF_INFO`).
- Add a periodic timer (e.g. `std::thread::sleep` loop, or `tokio` scheduled task) to
  re-run the update check every N minutes/hours.
- Register the binary in `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` for autostart.
- The `winget update` command name may vary (`upgrade` is the canonical alias); confirm
  the right sub-command to call.

---

## Parser notes

`winget upgrade` prints a fixed-width table.  The header line names the columns; the parser
locates each column by byte offset of the column header word (`find("name")`,
`find("id")`, etc.) and then slices each data row at those offsets.

**Edge cases to be aware of:**
- The output may contain a BOM or ANSI escape sequences on some Windows versions.
- Column widths vary with the longest entry; the offset approach handles this correctly.
- The last line of useful output is followed by a summary line containing `"upgrades available"`,
  which `is_end_row()` uses as a sentinel.
- Multi-byte (non-ASCII) characters in package names may shift byte offsets; the current
  code does not handle this.

Sample output (from `README.md`) can be used as the input for unit tests.

---

## Coding conventions

- Rust 2024 edition.
- Keep `main.rs` focused; move sub-systems into separate modules as they grow.
- Prefer returning `Result` with descriptive errors over `unwrap`/`expect` in library code.
- Unit tests live in the same file as the code they test (`#[cfg(test)]` modules).

---

## Build & test

```bash
# Build (requires Windows target toolchain when compiling Windows-specific APIs)
cargo build

# Run unit tests (parser tests can run on any platform)
cargo test
```
