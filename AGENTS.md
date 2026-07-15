# Repository Guidelines

## Project Structure & Module Organization

This is a Rust 2024 Windows tray application. Keep the entry point small in `src/main.rs`; reusable modules are exported through `src/lib.rs`.

- `src/app.rs`: tray UI, scheduling, notifications, and Windows autostart.
- `src/process.rs`: `winget` process invocation.
- `src/parser.rs`: fixed-width output parsing and package data types.
- `src/error.rs`: typed application, process, and parsing errors.
- `.github/workflows/ci.yml`: Linux tests and Windows release build.
- `assets/`: branded tray and installer icons.
- `installer/`: Inno Setup definition and PowerShell build helper.
- `README.md`: supported behavior and representative `winget upgrade` output.

Unit tests live beside their implementation in `#[cfg(test)]` modules.

## Build, Test, and Development Commands

- `cargo build`: compile a debug build for the host platform.
- `cargo build --release --target x86_64-pc-windows-msvc`: produce the Windows release executable used by CI.
- `cargo test --all-targets`: run parser, process, library, and binary tests.
- `cargo fmt --all -- --check`: verify standard Rust formatting without modifying files.
- `cargo clippy --all-targets --all-features -- -D warnings`: reject lint warnings.
- `cargo run`: launch the application; meaningful tray behavior requires Windows and an installed `winget`.
- `.\installer\build.ps1`: build the release binary and per-user installer; requires Inno Setup 6.

Before submitting changes, run formatting, tests, Clippy, and an appropriate build.

## Coding Style & Naming Conventions

Use `rustfmt` defaults (four-space indentation). Follow Rust naming conventions: `snake_case` for functions and modules, `CamelCase` for types and enum variants, and `SCREAMING_SNAKE_CASE` for constants. Prefer small modules, typed `Result` errors, and `?` over `unwrap` or `expect` in production code. Gate Windows-specific code and dependencies with `cfg(windows)`.

## Testing Guidelines

Use Rust's built-in test framework. Name tests after behavior, such as `test_parse_winget_raw_bad_source`. Add regression tests for parser edge cases, process failures, and platform-specific branches. Tests that can run cross-platform should remain outside OS-specific configuration. No coverage threshold is enforced, but new behavior should be directly exercised.

## Commit & Pull Request Guidelines

Recent commits use short, imperative summaries such as `Tidy tray interval constants` and `Address validation feedback`. Keep each commit focused and avoid mixing unrelated cleanup.

Pull requests should explain the user-visible effect, summarize implementation choices, and list validation commands. Link relevant issues. Include screenshots only for tray or notification changes. Ensure both the Linux test job and Windows release build pass before merge.

## Security & Configuration Tips

Do not commit machine-specific paths, registry exports, credentials, or generated `target/` contents. Changes to the `HKCU` autostart key or process arguments should be reviewed for quoting and failure handling.
