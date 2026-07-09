# winget-automation

A Windows system-tray application written in Rust that automatically detects outdated packages managed by [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/) and surfaces them as a notification.

## Features

| Feature | Status |
|---|---|
| Invoke `winget upgrade` and capture output | ✅ Implemented |
| Parse the fixed-width table into typed structs | ✅ Implemented |
| Typed error hierarchy (`WingetError`, `ParseWingetError`) | ✅ Implemented |
| Windows system-tray icon | 🔜 Planned |
| Balloon / toast notification | 🔜 Planned |
| Periodic update check (configurable interval) | 🔜 Planned |
| Autostart via `HKCU\…\Run` registry key | 🔜 Planned |

## Requirements

- Windows 10 or 11
- [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/) installed (comes with the App Installer from the Microsoft Store)
- Rust toolchain targeting `x86_64-pc-windows-msvc`

## Building

```bash
# Debug build
cargo build

# Release build for Windows
cargo build --release --target x86_64-pc-windows-msvc
```

## Running tests

Parser and process-level unit tests can run on any platform:

```bash
cargo test
```

## Repository layout

```
src/
  main.rs      — entry point: runs winget, parses output, prints results
  lib.rs       — re-exports the public modules
  error.rs     — typed error hierarchy (WingetError, ParseWingetError, …)
  parser.rs    — fixed-width table parser + WingetUpdateOutput / PackageSource types
  process.rs   — winget process invocation
Cargo.toml     — package manifest and dependencies
```

## Parser notes

`winget upgrade` prints a fixed-width table whose column offsets are determined by the
position of each header word in the header line.  The parser locates those offsets and
slices each data row accordingly.  The summary line (`"N upgrades available."`) acts as
a sentinel to stop parsing.

### Sample `winget upgrade` output

```
Name                                                        Id                               Version        Available     Source
---------------------------------------------------------------------------------------------------------------------------------
7-Zip 26.01 (x64)                                           7zip.7zip                        26.01          26.02         winget
AWS Command Line Interface v2                               Amazon.AWSCLI                    2.35.9.0       2.35.15       winget
AWS SAM Command Line Interface                              Amazon.SAM-CLI                   1.158.0        1.163.0       winget
balenaEtcher                                                Balena.Etcher                    2.1.4          2.1.6         winget
calibre 64bit                                               calibre.calibre                  9.9.0          9.11.0        winget
Docker Desktop                                              Docker.DockerDesktop             4.78.0         4.80.0        winget
Git                                                         Git.Git                          2.54.0         2.55.0        winget
GitHub CLI                                                  GitHub.cli                       2.95.0         2.96.0        winget
GNU Privacy Guard                                           GnuPG.GnuPG                      2.5.18         2.5.21        winget
Google Chrome                                               Google.Chrome.EXE                149.0.7827.201 150.0.7871.47 winget
Microsoft Edge                                              Microsoft.Edge                   149.0.4022.98  150.0.4078.48 winget
Microsoft ODBC Driver 17 for SQL Server                     Microsoft.msodbcsql.17           17.10.6.1      17.11.1.1     winget
Microsoft Visual Studio 2010 Tools for Office Runtime (x64) Microsoft.VSTOR                  10.0.60910     10.0.60917    winget
MiKTeX                                                      MiKTeX.MiKTeX                    24.4           25.12         winget
Oh My Posh                                                  XP8K0HKJFRXGCK                   29.17.0        29.20.0       msstore
QEMU                                                        SoftwareFreedomConservancy.QEMU  10.2.0         11.0.50       winget
Visual Studio Community 2026                                Microsoft.VisualStudio.Community 18.7.1         18.7.2        winget
Wacom Tablet                                                Wacom.WacomTabletDriver          6.4.12-3       6.4.13-4      winget
Windows Subsystem for Linux                                 Microsoft.WSL                    2.7.8.0        2.7.10        winget
Zed                                                         ZedIndustries.Zed                1.7.2          1.9.0         winget
20 upgrades available.
```

## License

See [LICENSE](LICENSE).
